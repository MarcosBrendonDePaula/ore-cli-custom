import { PrismaClient } from '@prisma/client';

interface Hash {
  id: string;
  hash: string;
  difficulty: number;
  minerAddress: string;
  status: string;
  createdAt: Date;
  updatedAt: Date;
}

const prisma = new PrismaClient();

async function getHashes(): Promise<Hash[]> {
  const hashes = await prisma.hash.findMany({
    orderBy: {
      difficulty: 'desc'
    }
  });
  return hashes;
}

export default async function HashesPage() {
  const hashes = await getHashes();

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold mb-6">Mining Pool Hashes</h1>
      <div className="overflow-x-auto">
        <table className="min-w-full bg-white dark:bg-gray-800 shadow-md rounded-lg">
          <thead className="bg-gray-50 dark:bg-gray-700">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Hash</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Difficulty</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Miner</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Status</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Time</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-gray-600">
            {hashes.map((hash: Hash) => (
              <tr key={hash.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                <td className="px-6 py-4 whitespace-nowrap text-sm font-mono">
                  {hash.hash.substring(0, 16)}...
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm">
                  {hash.difficulty}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-mono">
                  {hash.minerAddress.substring(0, 8)}...
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full 
                    ${hash.status === 'PENDING' ? 'bg-yellow-100 text-yellow-800' : 
                      hash.status === 'CONFIRMED' ? 'bg-green-100 text-green-800' : 
                      'bg-red-100 text-red-800'}`}>
                    {hash.status.toLowerCase()}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                  {new Date(hash.createdAt).toLocaleString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
