import { ApplicationConfig } from "@angular/core";
import { provideRouter } from "@angular/router";

import { routes } from "./app.routes";
import {ListenerService} from "./listener.service";

export const appConfig: ApplicationConfig = {
    providers: [provideRouter(routes),{provide:ListenerService,useValue:ListenerService.instance}],
};
