import {Component,ComponentRef, Input,Type,OnInit} from "@angular/core";
import {WindowAsmComponent} from "./window-asm/window-asm.component";
import {WindowHomeComponent} from "./window-home/window-home.component";
import {WindowCppComponent} from "./window-cpp/window-cpp.component";
import {invoke} from "@tauri-apps/api/core"
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
    protected mcuList:string[]|null = null;
    protected activeWindow:string|null = null;

    ngOnInit(){
        this.get_mcu_list();
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

    private get_mcu_list(){
        console.log("get_mcu_list");
        invoke("get_mcu_list").then(data=>{
            let data_new = data as Array<string>
            //console.log(data_new);
            this.mcuList = data_new.sort();
        })
    }

}