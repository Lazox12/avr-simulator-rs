import {Type} from "@angular/core";
import {WindowAsmComponent} from "./window-asm/window-asm.component";
import {WindowCppComponent} from "./window-cpp/window-cpp.component";

export interface AppWindow {
    title: string;
    key: string;
    component: Type<any>;
}

let test: AppWindow = {title:"fgae",key:"faeg",component:WindowAsmComponent};
let a:Type<any> = test.component;

export let windows: AppWindow[] = [
    { title: 'asm', key: 'asm', component: WindowAsmComponent },
    { title: 'cpp', key: 'cpp', component: WindowCppComponent },
];

export function getComponentList(){
    let componentList:Type<any>[] = [];
    windows.forEach(window => {componentList.push(window.component);});
    return componentList;
}

