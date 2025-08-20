import {Component,ComponentRef, Input,Type,OnInit} from "@angular/core";

export interface AppWindow{
    name:string;
    key:string;
    path:string;
    component:Type<any>|null;
}

@Component({
    selector: 'app-window-handler',
    standalone: false,
    templateUrl: "window-handler.component.html",
})
export class WindowHandlerComponent implements OnInit{

    windowList:AppWindow[]= [
        {name:"home",key:"home",path:"./window-home/window-home.component",component:null},
        {name:"disassembly",key:"asm",path:"./window-asm/window-asm.component",component:null},
        {name:"cpp",key:"cpp",path:"./window-cpp/window-cpp.component",component:null},
    ];
    activeWindow:Type<any>|null = null;

    ngOnInit(): void {
        this.loadComponents().then(()=>{})
    }

    async loadComponents(){
        for (const item of this.windowList) {
            let c = await import(item.path);
        }
    }

    async setActive(key: string) {
        let win = this.windowList.find((item:AppWindow)=>item.key === key);
        if (!win){
            console.warn(`Window handler does not exist!`);
            return;
        }
        this.activeWindow = win.component;
    }

}