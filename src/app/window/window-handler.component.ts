import {Component,ComponentRef, Input,Type,OnInit} from "@angular/core";
import {WindowAsmComponent} from "./window-asm/window-asm.component";
import {WindowHomeComponent} from "./window-home/window-home.component";
import {WindowCppComponent} from "./window-cpp/window-cpp.component";
import {ListenerService} from "../listener.service";
import {TauriEvent} from "@tauri-apps/api/event"
export interface AppWindow{
    name:string;
    key:string;
    path:string;
    component: Promise<any> | undefined;
}

@Component({
    selector: 'app-window-handler',
    standalone: true,
    imports: [WindowAsmComponent,WindowHomeComponent,WindowCppComponent],
    templateUrl: "window-handler.component.html",
    styleUrls: ["window-handler.component.css"]
})
export class WindowHandlerComponent implements OnInit {

    protected activeWindow:string|null = null;

    ngOnInit(){
        let w = localStorage.getItem('window-handler-active');
        if(w===null){
            return;
        }
        this.activeWindow = w;
        localStorage.removeItem('window-handler-active');

    }

    async setActive(key: string) {
        this.activeWindow = key;
    }


}