import { Component} from '@angular/core';
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Event } from '@tauri-apps/api/event';
import {ListenerService} from "../../listener.service";
import {invoke} from "@tauri-apps/api/core";

type PartialInstruction={
    comment: String,
    operands: Operand[]|null,
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
type Operand= {
    name: String,
    constraint:String,
    value: number,
    operandInfo: OperandInfo|null,

}
type OperandInfo = {
    registerName:String,
    registerMask:String,
    description:String,
}

@Component({
    selector: 'app-window-asm',
    standalone: true,
    templateUrl: './window-asm.component.html',
    styleUrl: './window-asm.component.css'
})
export class WindowAsmComponent {
    private hovertimeout :any|null = null;

    protected popupData:String|null = null;
    protected instructions : PartialInstruction[]|null = null;
    constructor() {
        console.log('WindowAsmComponent initialized');
        let a = localStorage.getItem('asm-data');
        if(a===null){
            return
        }
        this.instructions = JSON.parse(a);
        console.log(this.instructions);
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
    protected printInstructionPopup(opcode_id:number):String{
        let i = this.getInstruction(opcode_id);
        return `description: ${i.description}<br>action: ${i.action}`;

    }
    protected printOperandPopup(operand:Operand):String{
        return `value: ${operand.value.toString(16)} <br>register mask: ${operand.operandInfo?.registerMask} <br>description: ${operand.operandInfo?.description}`;
    }
    protected printOperandValue(op:Operand):String{
        if (op.operandInfo!=null){
            console.log(op.operandInfo);
            return op.operandInfo.registerName
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
    protected renderOperand(op:Operand[]|null) {
        let def:Operand = {
            name: "",
            constraint:"-1",
            value: 0,
            operandInfo: null,
        }
        if (op ===null){
            op = new Array<Operand>();
        }
        const result = [...op];
        while (result.length < 3) {
            result.push(def);
        }
        return result.slice(0, 3); // ensure only 3
    }

    static asmUpdateCallback(event:Event<PartialInstruction[]>){
        console.log("asmUpdateCallback");
        localStorage.removeItem('asm-data');
        console.log(event);
        localStorage.setItem('asm-data', JSON.stringify(event.payload));
        localStorage.setItem('window-handler-active', 'asm');
        window.location.reload();
    }
    static closeCallback(event:Event<String>){
        localStorage.removeItem('asm-data');
    }


    mouseEnter(data:String,address:number,column:number):void{
        this.hovertimeout = setTimeout(() => {
            console.log("test")
            this.popupData = data;
            let f = document.getElementById("asm-table-col-"+column+"-"+address);
            let pop = document.getElementById("asm-popup");
            if(f===null || pop===null){
                console.error(data,address,column);
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
        this.popupData = null;
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
ListenerService.instance.subscribe<String>('tauri://close-requested', WindowAsmComponent.closeCallback);

