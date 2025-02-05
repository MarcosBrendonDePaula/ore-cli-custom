import { prisma } from '@/lib/prisma';
import { revalidatePath } from 'next/cache';

async function getSettings() {
  const settings = await prisma.settings.findFirst() || {
    validatorAddress: process.env.VALIDATOR_ADDRESS || '',
    minDifficulty: 16,
    wsPort: parseInt(process.env.WS_PORT || '3001'),
  };
  return settings;
}

async function updateSettings(formData: FormData) {
  'use server';
  
  const validatorAddress = formData.get('validatorAddress') as string;
  const minDifficulty = parseInt(formData.get('minDifficulty') as string);
  const wsPort = parseInt(formData.get('wsPort') as string);

  await prisma.settings.upsert({
    where: { id: 1 },
    create: {
      validatorAddress,
      minDifficulty,
      wsPort,
    },
    update: {
      validatorAddress,
      minDifficulty,
      wsPort,
    },
  });

  revalidatePath('/settings');
}

export default async function Settings() {
  const settings = await getSettings();

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Pool Settings</h1>
      
      <form action={updateSettings} className="space-y-4">
        <div>
          <label htmlFor="validatorAddress" className="block text-sm font-medium text-gray-700">
            Validator Address
          </label>
          <input
            type="text"
            id="validatorAddress"
            name="validatorAddress"
            defaultValue={settings.validatorAddress}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label htmlFor="minDifficulty" className="block text-sm font-medium text-gray-700">
            Minimum Difficulty
          </label>
          <input
            type="number"
            id="minDifficulty"
            name="minDifficulty"
            defaultValue={settings.minDifficulty}
            min="1"
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          />
        </div>

        <div>
          <label htmlFor="wsPort" className="block text-sm font-medium text-gray-700">
            WebSocket Port
          </label>
          <input
            type="number"
            id="wsPort"
            name="wsPort"
            defaultValue={settings.wsPort}
            min="1"
            max="65535"
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          />
        </div>

        <button
          type="submit"
          className="inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
        >
          Save Settings
        </button>
      </form>
    </div>
  );
}
