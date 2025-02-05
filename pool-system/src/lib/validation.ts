import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { PrismaClient } from '@prisma/client';
import * as bs58 from 'bs58';
import * as ore from 'ore-api';

const prisma = new PrismaClient();

interface ValidationConfig {
  rpcUrl: string;
  keypairPath: string;
  minDifficulty: number;
  maxBatchSize: number;
}

export class HashValidator {
  private connection: Connection;
  private keypair: Keypair;
  private config: ValidationConfig;

  constructor(config: ValidationConfig) {
    this.connection = new Connection(config.rpcUrl);
    // Load keypair from file
    const keypairData = require('fs').readFileSync(config.keypairPath);
    this.keypair = Keypair.fromSecretKey(new Uint8Array(JSON.parse(keypairData)));
    this.config = config;
  }

  async validateAndSubmit() {
    // Get pending hashes above minimum difficulty
    const pendingHashes = await prisma.hash.findMany({
      where: {
        status: 'PENDING',
        difficulty: {
          gte: this.config.minDifficulty
        }
      },
      orderBy: {
        difficulty: 'desc'
      },
      take: this.config.maxBatchSize
    });

    if (pendingHashes.length === 0) {
      console.log('No pending hashes to validate');
      return;
    }

    // Process each hash
    for (const hash of pendingHashes) {
      try {
        // Create transaction
        const tx = new Transaction();
        
        // Find available bus
        const bus = await this.findAvailableBus();
        
        // Add auth instruction
        tx.add(ore.instruction.auth(
          new PublicKey(hash.minerAddress)
        ));
        
        // Add mine instruction with the hash data
        tx.add(ore.instruction.mine(
          new PublicKey(hash.minerAddress),
          this.keypair.publicKey,
          bus,
          {
            hash: bs58.decode(hash.hash),
            nonce: Buffer.from(hash.nonce || '', 'hex')
          }
        ));

        // Sign and send transaction
        const signature = await this.connection.sendTransaction(tx, [this.keypair]);
        
        // Wait for confirmation
        const confirmation = await this.connection.confirmTransaction(signature);
        
        if (confirmation.value.err) {
          // Update hash status to rejected if transaction failed
          await prisma.hash.update({
            where: { id: hash.id },
            data: { 
              status: 'REJECTED',
              error: JSON.stringify(confirmation.value.err)
            }
          });
          console.log(`Hash ${hash.id} rejected: ${confirmation.value.err}`);
        } else {
          // Update hash status to confirmed
          await prisma.hash.update({
            where: { id: hash.id },
            data: { 
              status: 'CONFIRMED',
              signature: signature
            }
          });
          console.log(`Hash ${hash.id} confirmed with signature: ${signature}`);
        }
      } catch (error: any) {
        console.error(`Error validating hash ${hash.id}:`, error);
        await prisma.hash.update({
          where: { id: hash.id },
          data: { 
            status: 'REJECTED',
            error: error.message
          }
        });
      }
    }
  }

  private async findAvailableBus(): Promise<PublicKey> {
    // Get all bus accounts
    const busAccounts = await Promise.all(
      ore.consts.BUS_ADDRESSES.map(addr => 
        this.connection.getAccountInfo(new PublicKey(addr))
      )
    );

    // Find first available bus (not null account)
    const availableBus = ore.consts.BUS_ADDRESSES.find((_, i) => 
      busAccounts[i] !== null
    );

    if (!availableBus) {
      throw new Error('No available bus found');
    }

    return new PublicKey(availableBus);
  }
}
