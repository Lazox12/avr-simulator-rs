import {Component, Input,Type} from "@angular/core";
import {WindowAsmComponent} from "./window-asm/window-asm.component";
import {WindowCppComponent} from "./window-cpp/window-cpp.component";
import {getComponentList,windows} from "./window-list.template";


@Component({
    selector: 'app-window-handler',
    standalone: true,
    templateUrl: "window-handler.component.html",
    imports: getComponentList()
})
export class WindowHandlerComponent {

    activeWindowKey: string | null = 'home';

    setActive(key: string) {
        this.activeWindowKey = key;
    }
}