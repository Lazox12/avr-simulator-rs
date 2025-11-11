import {Component, OnInit} from '@angular/core';
import {execute} from "../../command.service";

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
    private async get_mcu_list(){
        this.mcuList = await execute<Array<string>>("get_mcu_list");
    }
}
