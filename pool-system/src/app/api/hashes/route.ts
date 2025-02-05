import { NextResponse } from 'next/server';
import { PrismaClient } from '@prisma/client';

const prisma = new PrismaClient();

export async function POST(request: Request) {
  try {
    const body = await request.json();
    
    if (!body || typeof body !== 'object') {
      return NextResponse.json(
        { error: 'Invalid request body' },
        { status: 400 }
      );
    }

    const { hash, difficulty, minerAddress } = body;

    if (!hash || !difficulty || !minerAddress) {
      return NextResponse.json(
        { error: 'Missing required fields' },
        { status: 400 }
      );
    }

    const savedHash = await prisma.hash.create({
      data: {
        hash: String(hash),
        difficulty: Number(difficulty),
        minerAddress: String(minerAddress),
        createdAt: new Date(),
        status: 'PENDING'
      }
    });

    return NextResponse.json(savedHash);
  } catch (error) {
    console.error('Error saving hash:', error);
    return NextResponse.json(
      { error: 'Failed to save hash' },
      { status: 500 }
    );
  }
}

export async function GET() {
  try {
    const hashes = await prisma.hash.findMany({
      orderBy: {
        difficulty: 'desc'
      }
    });

    return NextResponse.json(hashes);
  } catch (error) {
    console.error('Error fetching hashes:', error);
    return NextResponse.json(
      { error: 'Failed to fetch hashes' },
      { status: 500 }
    );
  }
}
