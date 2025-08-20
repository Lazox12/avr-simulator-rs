import {Component,ComponentRef, Input,Type,OnInit} from "@angular/core";

export interface AppWindow{
    name:string;
    key:string;
    path:string;
    component: Promise<any> | undefined;
}

@Component({
    selector: 'app-window-handler',
    standalone: false,
    templateUrl: "window-handler.component.html",
})
export class WindowHandlerComponent implements OnInit{

    windowList:AppWindow[]= [
        {name:"home",key:"home",path:"./window-home/window-home.component",component:undefined},
        {name:"disassembly",key:"asm",path:"./window-asm/window-asm.component",component:undefined},
        {name:"cpp",key:"cpp",path:"./window-cpp/window-cpp.component",component:undefined},
    ];
    activeWindow:Promise<any> | undefined = undefined;

    ngOnInit(): void {
        this.loadComponents().then(()=>{})
    }

    async loadComponents(){
        for (const item of this.windowList) {
            console.log(item);
            item.component = import(/* webpackPrefetch: 1 */item.path).then(i => i.InformationComponent);
            console.log(item);
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