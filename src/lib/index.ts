// place files you want to import through the `$lib` alias in this folder.
import { invoke } from '@tauri-apps/api/tauri';

export async function doStuff() {
	let s = await fetchData();
	console.log(s);
	return s;
}

async function fetchData(): Promise<string> {
	return await invoke('my_custom_command', { value: 'fetchi' });
}

function Sleep(milliseconds: number) {
	return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
