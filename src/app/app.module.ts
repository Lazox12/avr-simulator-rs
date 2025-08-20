import {NgModule} from '@angular/core'
import {BrowserModule} from '@angular/platform-browser'
import {AppComponent} from "./app.component";
import {WindowHandlerComponent} from "./window/window-handler.component";
import {WindowHomeComponent} from "./window/window-home/window-home.component";
import {WindowCppComponent} from "./window/window-cpp/window-cpp.component";
import {WindowAsmComponent} from "./window/window-asm/window-asm.component";

@NgModule({
    declarations: [AppComponent,WindowHandlerComponent],
    imports: [
        BrowserModule,
    ],
    bootstrap: [AppComponent]
})
export class AppModule {}
