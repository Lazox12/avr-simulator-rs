import { Component,NO_ERRORS_SCHEMA } from '@angular/core';
import {WindowHandlerComponent} from "./window/window-handler.component";

@Component({
  selector: 'app-root',
  standalone:true,
  imports:[WindowHandlerComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
  schemas:[NO_ERRORS_SCHEMA]
})
export class AppComponent {

}
