import { ProgressToken, WorkDoneProgress } from "vscode-languageserver-protocol";

export interface ProgressParams {
    /**
     * The progress token provided by the client or server.
     */
    token: ProgressToken;
    /**
     * The progress data.
     */
    value: typeof WorkDoneProgress.type._pr;
}

