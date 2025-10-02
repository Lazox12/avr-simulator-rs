import { Component} from '@angular/core';
import { Event } from '@tauri-apps/api/event';
import {ListenerService} from "../../listener.service";

type PartialInstruction={
    comment: String,
    operands: String[]|null,
    address: number,
    name:String,
}


@Component({
    selector: 'app-window-asm',
    standalone: true,
    templateUrl: './window-asm.component.html',
    styleUrl: './window-asm.component.css'
})
export class WindowAsmComponent {
    private hovertimeout :any|null = null;

    protected popupInst:PartialInstruction|null = null;
    protected instructions : PartialInstruction[]|null = null;
    constructor() {
        console.log('WindowAsmComponent initialized');
        let a = localStorage.getItem('asm-data');
        if(a===null){
            return
        }
        this.instructions = JSON.parse(a);
    }

    static asmUpdateCallback(event:Event<PartialInstruction[]>){
        localStorage.removeItem('asm-data');
        console.log(event);
        localStorage.setItem('asm-data', JSON.stringify(event.payload));
        localStorage.setItem('window-handler-active', 'asm');
        window.location.reload();
    }


    mouseEnter(inst:PartialInstruction){
        this.hovertimeout = setTimeout(() => {
            this.popupInst = inst;
            let f = document.getElementById("asm-table-col-"+inst.address);
            let pop = document.getElementById("asm-popup");
            if(f===null || pop===null){
                return;
            }
            let rect = f.getBoundingClientRect();
            console.log(rect);
            pop.style.top = (rect.top+rect.height) + "px";
            pop.style.left = rect.left + "px";
            pop.style.visibility = "visible";
        }, 1500)
    }
    mouseLeave(){
        if (this.hovertimeout === null) {
            return;
        }
        clearTimeout(this.hovertimeout);
        let pop = document.getElementById("asm-popup");
        if (pop === null) {
            return;
        }
        pop.style.visibility = "hidden";
        this.popupInst = null;
    }
    protected applyChanges(){
        console.log("applying changes...");
        let el = document.getElementById('asm-apply-changes-button')
        if (el === null) {
            return;
        }
        (el as HTMLButtonElement).disabled = true;
    }
    protected clearTable(){
        this.instructions = [];
        localStorage.removeItem('asm-data');
    }
    protected tableOnKeyUp(event: KeyboardEvent){
        let el = document.getElementById('asm-apply-changes-button')
        if (el === null) {
            return;
        }
        (el as HTMLButtonElement).disabled = false;
    }
}
ListenerService.instance.subscribe<PartialInstruction[]>('asm-update', WindowAsmComponent.asmUpdateCallback);

