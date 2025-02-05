import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';

// Simulated ore-api constants and functions for validation
export const consts = {
    BUS_ADDRESSES: [
        'BUS1111111111111111111111111111111111111111',
        'BUS2222222222222222222222222222222222222222',
        'BUS3333333333333333333333333333333333333333',
    ],
    PROGRAM_ID: new PublicKey('ore11111111111111111111111111111111111111111'),
};

export const instruction = {
    auth(minerAddress: PublicKey): TransactionInstruction {
        return new TransactionInstruction({
            programId: consts.PROGRAM_ID,
            keys: [
                { pubkey: minerAddress, isSigner: false, isWritable: true },
            ],
            data: Buffer.from([0]), // Auth instruction
        });
    },

    mine(
        minerAddress: PublicKey,
        validatorAddress: PublicKey,
        busAddress: PublicKey,
        solution: { hash: Buffer, nonce: Buffer }
    ): Transaction {
        const tx = new Transaction();
        
        // Add auth instruction
        tx.add(this.auth(minerAddress));
        
        // Add mine instruction
        tx.add(new TransactionInstruction({
            programId: consts.PROGRAM_ID,
            keys: [
                { pubkey: minerAddress, isSigner: false, isWritable: true },
                { pubkey: validatorAddress, isSigner: true, isWritable: false },
                { pubkey: busAddress, isSigner: false, isWritable: true },
            ],
            data: Buffer.concat([
                Buffer.from([1]), // Mine instruction
                solution.hash,
                solution.nonce,
            ]),
        }));

        return tx;
    },
};
