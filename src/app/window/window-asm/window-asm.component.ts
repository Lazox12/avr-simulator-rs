import { Component,OnInit,inject } from '@angular/core';
import { Event } from '@tauri-apps/api/event';
import {ListenerService} from "../../listener.service";

type Instruction={
    opcode: RawInst,
    operands: Operand[]|null,
    address: number,
    rawOpcode: number,
    comment: String,
}
type RawInst = {
    opcode:String,
    len:number,
    name:String,
    constraints:ConstraintMap[] | null,
    binMask:number,
    binOpcode:number,
    action:String,
    description:String,
}
type ConstraintMap = {
    map:number,
    constraint:string,
}
type Operand = {
    name: String,
    constraint:String,
    value: number,
}



@Component({
    selector: 'app-window-asm',
    standalone: true,
    templateUrl: './window-asm.component.html',
    styleUrl: './window-asm.component.css'
})
export class WindowAsmComponent {
    private hovertimeout :any|null = null;
    private listener = inject(ListenerService);
    protected popupInst:Instruction|null = null;
    protected instructions : Instruction[]|null = null;
    constructor() {
        console.log('WindowAsmComponent initialized');
        let a = localStorage.getItem('asm-data');
        if(a===null){
            return
        }
        this.instructions = JSON.parse(a);
    }

    static asmUpdateCallback(event:Event<Instruction[]>){
        localStorage.removeItem('asm-data');
        console.log(event);
        localStorage.setItem('asm-data', JSON.stringify(event.payload));
    }
    printOperand(op:Operand|undefined):string{
        if(op === undefined){
            return '';
        }
        switch(op.constraint){
            case "r":{
                return "r"+String(op.value);
            }
            case "d":{
                return "r"+String(op.value);
            }
            case "v":{
                return "r"+String(op.value);
            }
            case "a":{
                return "r"+String(op.value);
            }
            case "w":{
                return "r"+String(op.value);
            }
            case "e":{
                switch (op.value){
                    case 3: return "X";
                    case 2: return "Y";
                    case 0: return "Z";
                    default: return "cant deocde"
                }
            }
            case "b":{
                switch (op.value){
                    case 0: return "Z";
                    case 1: return "Y";
                    default: return "cant deocde"
                }
            }
            case "z":{
                if(op.value!=0){
                    return "Z+";
                }
                return "";
            }
            case "M":{
                return "0x"+op.value.toString(16);
            }
            case "n":{
                return "0x"+op.value.toString(16);
            }
            case "s":{
                return "0x"+op.value.toString(16);
            }
            case "P":{
                return "0x"+op.value.toString(16);
            }
            case "p":{
                return "0x"+op.value.toString(16);
            }
            case "K":{
                return "0x"+op.value.toString(16);
            }
            case "i":{
                return "0x"+op.value.toString(16);
            }
            case "j":{
                return "0x"+op.value.toString(16);
            }
            case "l":{
                return "."+String(op.value);
            }
            case "L":{
                return "."+String(op.value);
            }
            case "h":{
                return "0x"+op.value.toString(16);
            }
            case "S":{
                return "0x"+op.value.toString(16);
            }
            case "E":{
                return "0x"+op.value.toString(16);
            }
            case "o":{
                return String(op.value);
            }
            default:{
                return 'error invild constraint:'+op.constraint;
            }
        }
    }

    mouseEnter(inst:Instruction){
        this.hovertimeout = setTimeout(() => {
            this.popupInst = inst;
            let f = document.getElementById("asm-table-col-"+inst.address);
            let pop = document.getElementById("asm-popup");
            if(f===null || pop===null){
                return;
            }
            let rect = f.getBoundingClientRect();
            pop.style.top = (rect.top) + "px";
            pop.style.left = rect.left + "px";
            pop.style.visibility = "visible";
        }, 3000)
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
}
ListenerService.instance.subscribe<Instruction[]>('asm-update', WindowAsmComponent.asmUpdateCallback);

