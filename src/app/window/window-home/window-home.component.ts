import { Component,OnInit } from '@angular/core';
import {invoke} from "@tauri-apps/api/core"

@Component({
  selector: 'app-window-home',
  standalone: true,
  templateUrl: './window-home.component.html',
  styleUrl: './window-home.component.css'
})
export class WindowHomeComponent implements OnInit{
    ngOnInit(): void {
        this.get_mcu_list()
    }
    protected mcuList:string[]|null = null;
    private get_mcu_list(){
        console.log("get_mcu_list");
        invoke("get_mcu_list").then(data=>{
            let data_new = data as Array<string>
            //console.log(data_new);
            this.mcuList = data_new.sort();
        })
    }
}
