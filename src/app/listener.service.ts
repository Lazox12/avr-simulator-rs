import { Event, EventName, EventCallback, listen } from "@tauri-apps/api/event";

type Listener<T> = {
    event: EventName;
    callback: EventCallback<T>;
};

export class ListenerService {
    static #instance: ListenerService;
    private constructor() {}
    public static get instance(): ListenerService {
        if (!ListenerService.#instance) {
            ListenerService.#instance = new ListenerService();
        }
        return ListenerService.#instance;
    }

    private listeners: Listener<any>[] = [];
    private subscribed: string[] = [];

    public subscribe<T>(event: EventName, callback: EventCallback<T>): void {
        this.listeners.push({ event, callback });
        if (!this.subscribed.includes(event)) {
            this.subscribed.push(event);
            listen<T>(event, (e) => this.callback<T>(e));
        }
        console.log("subscribed to", event);
    }

    private callback<T>(event: Event<T>) {
        console.log("received event:", event.event);
        this.listeners.forEach((listener) => {
            if (listener.event === event.event) {
                listener.callback(event);
            }
        });
    }
}
