/* tslint:disable */
/* eslint-disable */

/**
 * The main WASM-exported DAG editor.  One instance per canvas element.
 */
export class DagEditor {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Apply engine result JSON (updated DAG with `outputs` filled in) to the model.
     */
    apply_results_json(dag_json: string): boolean;
    can_redo(): boolean;
    can_undo(): boolean;
    clear(): void;
    /**
     * Copy the selected node's kind + config into the in-memory clipboard.
     * Copy selected node(s) + internal edges into clipboard.
     */
    copy_selected(): boolean;
    edge_count(): number;
    /**
     * Returns JSON of the topological execution order (array of node ids).
     */
    execution_order_json(): string;
    /**
     * Returns `true` on success.
     */
    from_json(json: string): boolean;
    /**
     * Returns the selected node's JSON (id + kind + config + outputs), or empty string.
     */
    get_selected_node_json(): string;
    /**
     * Handle a raw key press. Only `Delete`/`Backspace` (remove the current
     * selection) are handled here; every modifier-based shortcut (copy, paste,
     * select-all, layout, fit, …) is dispatched by the host page directly to
     * the dedicated method so the Ctrl/Meta state is actually honored.
     * Returns true if the DAG was mutated.
     */
    key_down(key: string): boolean;
    /**
     * Hierarchical auto-layout: left-to-right layering with vertical centering.
     */
    layout(): void;
    /**
     * Load a named template.  Returns true on success.
     */
    load_template(name: string): boolean;
    /**
     * Left-button press.  `button`: 0=left, 1=middle, 2=right.  `shift`: Shift key held.
     */
    mouse_down(sx: number, sy: number, button: number, shift: boolean): void;
    mouse_move(sx: number, sy: number): void;
    mouse_up(sx: number, sy: number): void;
    /**
     * Create a new editor attached to `<canvas id="{canvas_id}">`.
     */
    constructor(canvas_id: string);
    /**
     * Hit-test screen position → node id (u32::MAX if none).
     */
    node_at_screen(sx: number, sy: number): number;
    node_count(): number;
    /**
     * Returns {x,y,w,h} in screen pixels for the node bounding box, or "{}" if not found.
     */
    node_screen_rect(node_id: number): string;
    /**
     * Called when a palette drag is cancelled (dropped outside canvas).
     */
    palette_cancel(): void;
    /**
     * Called on every mousemove while dragging from the palette.
     */
    palette_drag_move(sx: number, sy: number): void;
    /**
     * Called when the user starts dragging a node type from the HTML palette.
     * `kind_str` must match the serde snake_case name of a `NodeKind` variant.
     */
    palette_drag_start(kind_str: string): void;
    /**
     * Called when the user releases a palette drag on the canvas.
     */
    palette_drop(sx: number, sy: number): void;
    /**
     * Returns JSON summary of available node kinds (palette + config schema).
     */
    static palette_json(): string;
    paste(): boolean;
    redo(): boolean;
    render(): void;
    /**
     * Draw a result chart for node `node_id` onto a separate canvas element.
     * Call after `apply_results_json()`. Returns false if the node has no outputs.
     */
    render_chart(node_id: number, chart_canvas_id: string): boolean;
    /**
     * Render a scaled-down overview of the DAG onto a separate minimap canvas.
     * Draws node rects, edges, and a viewport indicator box.
     */
    render_minimap(minimap_id: string, main_canvas_w: number, main_canvas_h: number): boolean;
    /**
     * Notify the editor that the canvas has been resized.
     */
    resize(): void;
    select_all_nodes(): void;
    selected_node_count(): number;
    /**
     * Returns the u32 ID of the selected node, or u32::MAX if nothing selected.
     */
    selected_node_id(): number;
    /**
     * Update the config of node `node_id` with a JSON object.  Returns true on success.
     */
    set_node_config(node_id: number, config_json: string): boolean;
    /**
     * Set a custom display label on a node (empty string clears it).
     */
    set_node_label(node_id: number, label: string): boolean;
    snap_enabled(): boolean;
    /**
     * JSON array of available template descriptors for the UI dropdown.
     */
    static template_catalog_json(): string;
    to_json(): string;
    /**
     * Toggle snap-to-grid.  `grid_size` = world-space grid size in pixels (e.g. 20).
     */
    toggle_snap(grid_size: number): boolean;
    undo(): boolean;
    wheel(delta_y: number, sx: number, sy: number): void;
    /**
     * Fit all nodes into view with padding.
     */
    zoom_fit(): void;
    zoom_in(): void;
    zoom_out(): void;
    zoom_percent(): number;
    zoom_reset(): void;
    /**
     * Fit view to selected nodes only; falls back to zoom_fit if nothing selected.
     */
    zoom_to_selected(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_dageditor_free: (a: number, b: number) => void;
    readonly dageditor_apply_results_json: (a: number, b: number, c: number) => number;
    readonly dageditor_can_redo: (a: number) => number;
    readonly dageditor_can_undo: (a: number) => number;
    readonly dageditor_clear: (a: number) => void;
    readonly dageditor_copy_selected: (a: number) => number;
    readonly dageditor_edge_count: (a: number) => number;
    readonly dageditor_execution_order_json: (a: number) => [number, number];
    readonly dageditor_from_json: (a: number, b: number, c: number) => number;
    readonly dageditor_get_selected_node_json: (a: number) => [number, number];
    readonly dageditor_key_down: (a: number, b: number, c: number) => number;
    readonly dageditor_layout: (a: number) => void;
    readonly dageditor_load_template: (a: number, b: number, c: number) => number;
    readonly dageditor_mouse_down: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly dageditor_mouse_move: (a: number, b: number, c: number) => void;
    readonly dageditor_mouse_up: (a: number, b: number, c: number) => void;
    readonly dageditor_new: (a: number, b: number) => [number, number, number];
    readonly dageditor_node_at_screen: (a: number, b: number, c: number) => number;
    readonly dageditor_node_count: (a: number) => number;
    readonly dageditor_node_screen_rect: (a: number, b: number) => [number, number];
    readonly dageditor_palette_cancel: (a: number) => void;
    readonly dageditor_palette_drag_move: (a: number, b: number, c: number) => void;
    readonly dageditor_palette_drag_start: (a: number, b: number, c: number) => void;
    readonly dageditor_palette_drop: (a: number, b: number, c: number) => void;
    readonly dageditor_palette_json: () => [number, number];
    readonly dageditor_paste: (a: number) => number;
    readonly dageditor_redo: (a: number) => number;
    readonly dageditor_render: (a: number) => void;
    readonly dageditor_render_chart: (a: number, b: number, c: number, d: number) => number;
    readonly dageditor_render_minimap: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly dageditor_resize: (a: number) => void;
    readonly dageditor_select_all_nodes: (a: number) => void;
    readonly dageditor_selected_node_count: (a: number) => number;
    readonly dageditor_selected_node_id: (a: number) => number;
    readonly dageditor_set_node_config: (a: number, b: number, c: number, d: number) => number;
    readonly dageditor_set_node_label: (a: number, b: number, c: number, d: number) => number;
    readonly dageditor_snap_enabled: (a: number) => number;
    readonly dageditor_template_catalog_json: () => [number, number];
    readonly dageditor_to_json: (a: number) => [number, number];
    readonly dageditor_toggle_snap: (a: number, b: number) => number;
    readonly dageditor_undo: (a: number) => number;
    readonly dageditor_wheel: (a: number, b: number, c: number, d: number) => void;
    readonly dageditor_zoom_fit: (a: number) => void;
    readonly dageditor_zoom_in: (a: number) => void;
    readonly dageditor_zoom_out: (a: number) => void;
    readonly dageditor_zoom_percent: (a: number) => number;
    readonly dageditor_zoom_reset: (a: number) => void;
    readonly dageditor_zoom_to_selected: (a: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
