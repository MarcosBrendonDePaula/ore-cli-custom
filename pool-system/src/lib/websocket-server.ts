import { WebSocketServer, WebSocket } from 'ws';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

interface MinerConnection {
  ws: WebSocket;
  address: string;
  isValidator: boolean;
}

type ErrorWithMessage = {
    message: string;
};

function isErrorWithMessage(error: unknown): error is ErrorWithMessage {
    return (
        typeof error === 'object' &&
        error !== null &&
        'message' in error &&
        typeof (error as Record<string, unknown>).message === 'string'
    );
}

function toErrorWithMessage(maybeError: unknown): ErrorWithMessage {
    if (isErrorWithMessage(maybeError)) return maybeError;
    try {
        return new Error(JSON.stringify(maybeError));
    } catch {
        return new Error(String(maybeError));
    }
}

export class PoolWebSocketServer {
  private wss: WebSocketServer;
  private miners: Map<string, MinerConnection> = new Map();
  private validatorAddress: string | null = null;

  constructor(port: number) {
    if (typeof window === 'undefined') {
      this.wss = new WebSocketServer({ port });
      this.setupWebSocketServer();
    } else {
      throw new Error('WebSocket server can only be initialized in Node.js environment');
    }

    // Load validator address from environment
    this.validatorAddress = process.env.VALIDATOR_ADDRESS || null;
    if (this.validatorAddress) {
      console.log(`Configured validator address: ${this.validatorAddress}`);
    }
  }

  private setupWebSocketServer() {
    this.wss.on('connection', (ws: WebSocket) => {
      let minerAddress: string;

      ws.on('message', async (message: Buffer) => {
        try {
          const data = JSON.parse(message.toString());
          
          switch (data.type) {
            case 'register':
              minerAddress = data.address;
              const isValidator = this.validatorAddress === minerAddress;
              this.miners.set(minerAddress, { 
                ws, 
                address: minerAddress,
                isValidator
              });
              ws.send(JSON.stringify({ 
                type: 'registered',
                isValidator
              }));
              console.log(`Miner ${minerAddress} connected${isValidator ? ' (Validator)' : ''}`);
              break;

            case 'submit_hash':
              await this.handleHashSubmission(data);
              break;

            case 'validation_result':
              if (!this.miners.get(minerAddress)?.isValidator) {
                ws.send(JSON.stringify({ 
                  type: 'error', 
                  message: 'Not authorized as validator' 
                }));
                return;
              }
              await this.handleValidationResult(data);
              break;
          }
        } catch (error) {
          console.error('Error processing message:', error);
          ws.send(JSON.stringify({ 
            type: 'error', 
            message: toErrorWithMessage(error).message
          }));
        }
      });

      ws.on('close', () => {
        if (minerAddress) {
          this.miners.delete(minerAddress);
          console.log(`Miner ${minerAddress} disconnected`);
        }
      });
    });
  }

  private async handleHashSubmission(data: any) {
    try {
      // Store hash in database
      const hash = await prisma.hash.create({
        data: {
          hash: data.hash,
          difficulty: data.difficulty,
          minerAddress: data.minerAddress,
          nonce: data.nonce,
          status: 'PENDING'
        }
      });

      // Forward to validator if available
      const validator = Array.from(this.miners.values()).find(m => m.isValidator);
      if (validator) {
        validator.ws.send(JSON.stringify({
          type: 'validate_hash',
          hashId: hash.id,
          hash: hash.hash,
          difficulty: hash.difficulty,
          minerAddress: hash.minerAddress,
          nonce: hash.nonce
        }));
      } else {
        console.log('No validator available to process hash');
        await prisma.hash.update({
          where: { id: hash.id },
          data: {
            status: 'PENDING',
            error: 'No validator available'
          }
        });
      }
    } catch (error) {
      console.error('Error handling hash submission:', error);
      throw error;
    }
  }

  private async handleValidationResult(data: any) {
    try {
      await prisma.hash.update({
        where: { id: data.hashId },
        data: {
          status: data.success ? 'CONFIRMED' : 'REJECTED',
          signature: data.signature,
          error: data.error
        }
      });

      // Notify the original miner
      const hash = await prisma.hash.findUnique({
        where: { id: data.hashId }
      });
      if (hash) {
        const miner = this.miners.get(hash.minerAddress);
        if (miner) {
          miner.ws.send(JSON.stringify({
            type: data.success ? 'hash_confirmed' : 'hash_rejected',
            hashId: data.hashId,
            signature: data.signature,
            error: data.error
          }));
        }
      }
    } catch (error) {
      console.error('Error handling validation result:', error);
    }
  }
}
