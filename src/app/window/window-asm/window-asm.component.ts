import { Component} from '@angular/core';
import { Event } from '@tauri-apps/api/event';
import {ListenerService} from "../../listener.service";
import {execute} from "../../command.service";
import {CommonModule} from "@angular/common";
type PartialInstruction={
    comment: string,
    commentDisplay:string,
    operands: Operand[]|null,
    address: number,
    opcodeId:number,
}
type RawInstruction = {
    opcode:string,
    len:number,
    name:string,
    constraints:ConstraintMap[]|null,
    binMask:number,
    binOpcode:number,
    action:string,
    description:string,
}
type ConstraintMap = {
    map:number,
    constraints:string,
}
type Operand= {
    name: string,
    constraint:string,
    value: number,
    operandInfo: OperandInfo|null,

}
type OperandInfo = {
    registerName:string,
    registerMask:string,
    description:string,
}

@Component({
    selector: 'app-window-asm',
    standalone: true,
    templateUrl: './window-asm.component.html',
    styleUrl: './window-asm.component.css',
    imports:[
        CommonModule,
    ]
})
export class WindowAsmComponent {
    private hovertimeout :any|null = null;

    protected popupData:string|null = null;
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
    protected async getInstruction(opcode_id: number): Promise<RawInstruction>{
        let x= localStorage.getItem("instruction-list");
        let i:RawInstruction[];
        if (x ===null){
            let a = await execute<RawInstruction[]>("get_instruction_list")
            if(a!==null){
                localStorage.setItem("instruction-list",JSON.stringify(a));
                i = a;
            }
            i = [];
        }else{
            i = JSON.parse(x)
        }
        let res = i.at(opcode_id);
        if(res!==undefined){
            return res;
        }
        if(opcode_id==999){
            return {
                action: "nothing",
                binMask: 0xffff,
                binOpcode: -1,
                constraints: null,
                description: "not a valid instruction, probably a constant stored in flash",
                len: -1,
                name: ".word",
                opcode: ".word"

            }
        }
        throw "error get_instruction_list";
    }
    protected async printInstructionPopup(opcode_id:number):Promise<string>{
        let i = await this.getInstruction(opcode_id);
        return `description: ${i.description}<br>action: ${i.action}`;

    }
    protected printOperandPopup(operand:Operand):string{
        return `value: ${operand.value.toString(16)} <br>register mask: ${operand.operandInfo?.registerMask} <br>description: ${operand.operandInfo?.description}`;
    }
    protected printOperandValue(op:Operand):string{
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
    static closeCallback(event:Event<string>){
        localStorage.removeItem('asm-data');
    }


    mouseEnter(data:Promise<string>|string,address:number,column:number):void{
        this.hovertimeout = setTimeout(async () => {
            console.log("test")
            if(typeof data !== "string"){
                data = await data;
            }
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

    printComment(i:PartialInstruction):string|undefined{
        switch (i.commentDisplay) {
            case "None":
                return "";
            case "Bin":
                return "0b"+parseInt(i.comment).toString(2);
            case "Dec":
                return parseInt(i.comment).toString(10);
            case "Oct":
                return "0c"+parseInt(i.comment).toString(8);
            case "Hex":
                return "0x"+parseInt(i.comment).toString(16);
            case "String":
                return i.comment;
        }
        return undefined;
    }

    protected readonly console = console;
}
ListenerService.instance.subscribe<PartialInstruction[]>('asm-update', WindowAsmComponent.asmUpdateCallback);
ListenerService.instance.subscribe<string>('tauri://close-requested', WindowAsmComponent.closeCallback);

