import { Component} from '@angular/core';
import { Event } from '@tauri-apps/api/event';
import {ListenerService} from "../../listener.service";
import {invoke} from "@tauri-apps/api/core";

type PartialInstruction={
    comment: String,
    operands: String[]|null,
    address: number,
    opcodeId:number,
}
type RawInstruction = {
    opcode:String,
    len:number,
    name:String,
    constraints:ConstraintMap[]|null,
    bin_mask:number,
    bin_opcode:number,
    action:String,
    description:String,
}
type ConstraintMap = {
    map:number,
    constraints:String,
}


@Component({
    selector: 'app-window-asm',
    standalone: true,
    templateUrl: './window-asm.component.html',
    styleUrl: './window-asm.component.css'
})
export class WindowAsmComponent {
    private hovertimeout :any|null = null;

    protected popupInst:RawInstruction|null = null;
    protected instructions : PartialInstruction[]|null = null;
    constructor() {
        console.log('WindowAsmComponent initialized');
        let a = localStorage.getItem('asm-data');
        if(a===null){
            return
        }
        this.instructions = JSON.parse(a);
    }
    protected getInstruction(opcode_id:number):RawInstruction{
        let x= localStorage.getItem("instruction-list");
        let i:RawInstruction[];
        if (x ===null){
            invoke("get_instruction_list").then(data=>{
                localStorage.setItem("instruction-list",JSON.stringify(data));
                localStorage.setItem('window-handler-active', 'asm');
                window.location.reload();
            })
            throw "error get_instruction_list";
        }else{
            i = JSON.parse(x)
        }
        return i[opcode_id];
    }
    static asmUpdateCallback(event:Event<PartialInstruction[]>){
        localStorage.removeItem('asm-data');
        console.log(event);
        localStorage.setItem('asm-data', JSON.stringify(event.payload));
        localStorage.setItem('window-handler-active', 'asm');
        window.location.reload();
    }


    mouseEnter(inst:RawInstruction,address:number):void{
        this.hovertimeout = setTimeout(() => {
            this.popupInst = inst;
            let f = document.getElementById("asm-table-col-"+address);
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

    protected readonly console = console;
}
ListenerService.instance.subscribe<PartialInstruction[]>('asm-update', WindowAsmComponent.asmUpdateCallback);

