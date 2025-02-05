-- CreateTable
CREATE TABLE "Hash" (
    "id" TEXT NOT NULL,
    "hash" TEXT NOT NULL,
    "difficulty" INTEGER NOT NULL,
    "minerAddress" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Hash_pkey" PRIMARY KEY ("id")
);
