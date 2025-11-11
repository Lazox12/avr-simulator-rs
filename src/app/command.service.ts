import {invoke,InvokeArgs} from '@tauri-apps/api/core'

export async function execute<T>(command: string, args?: InvokeArgs): Promise<T | null> {
    try {
        return await invoke<T>(command, args);
    } catch (err) {
        console.error(err);
        return null;
    }
}

