/* @ts-self-types="./hydrocomplete_dag.d.ts" */

/**
 * The main WASM-exported DAG editor.  One instance per canvas element.
 */
export class DagEditor {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DagEditorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dageditor_free(ptr, 0);
    }
    /**
     * Apply engine result JSON (updated DAG with `outputs` filled in) to the model.
     * @param {string} dag_json
     * @returns {boolean}
     */
    apply_results_json(dag_json) {
        const ptr0 = passStringToWasm0(dag_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_apply_results_json(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    can_redo() {
        const ret = wasm.dageditor_can_redo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    can_undo() {
        const ret = wasm.dageditor_can_undo(this.__wbg_ptr);
        return ret !== 0;
    }
    clear() {
        wasm.dageditor_clear(this.__wbg_ptr);
    }
    /**
     * Copy the selected node's kind + config into the in-memory clipboard.
     * Copy selected node(s) + internal edges into clipboard.
     * @returns {boolean}
     */
    copy_selected() {
        const ret = wasm.dageditor_copy_selected(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    edge_count() {
        const ret = wasm.dageditor_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns JSON of the topological execution order (array of node ids).
     * @returns {string}
     */
    execution_order_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.dageditor_execution_order_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns `true` on success.
     * @param {string} json
     * @returns {boolean}
     */
    from_json(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_from_json(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Returns the selected node's JSON (id + kind + config + outputs), or empty string.
     * @returns {string}
     */
    get_selected_node_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.dageditor_get_selected_node_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Handle a raw key press. Only `Delete`/`Backspace` (remove the current
     * selection) are handled here; every modifier-based shortcut (copy, paste,
     * select-all, layout, fit, …) is dispatched by the host page directly to
     * the dedicated method so the Ctrl/Meta state is actually honored.
     * Returns true if the DAG was mutated.
     * @param {string} key
     * @returns {boolean}
     */
    key_down(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_key_down(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Hierarchical auto-layout: left-to-right layering with vertical centering.
     */
    layout() {
        wasm.dageditor_layout(this.__wbg_ptr);
    }
    /**
     * Load a named template.  Returns true on success.
     * @param {string} name
     * @returns {boolean}
     */
    load_template(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_load_template(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Left-button press.  `button`: 0=left, 1=middle, 2=right.  `shift`: Shift key held.
     * @param {number} sx
     * @param {number} sy
     * @param {number} button
     * @param {boolean} shift
     */
    mouse_down(sx, sy, button, shift) {
        wasm.dageditor_mouse_down(this.__wbg_ptr, sx, sy, button, shift);
    }
    /**
     * @param {number} sx
     * @param {number} sy
     */
    mouse_move(sx, sy) {
        wasm.dageditor_mouse_move(this.__wbg_ptr, sx, sy);
    }
    /**
     * @param {number} sx
     * @param {number} sy
     */
    mouse_up(sx, sy) {
        wasm.dageditor_mouse_up(this.__wbg_ptr, sx, sy);
    }
    /**
     * Create a new editor attached to `<canvas id="{canvas_id}">`.
     * @param {string} canvas_id
     */
    constructor(canvas_id) {
        const ptr0 = passStringToWasm0(canvas_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_new(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        DagEditorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Hit-test screen position → node id (u32::MAX if none).
     * @param {number} sx
     * @param {number} sy
     * @returns {number}
     */
    node_at_screen(sx, sy) {
        const ret = wasm.dageditor_node_at_screen(this.__wbg_ptr, sx, sy);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    node_count() {
        const ret = wasm.dageditor_node_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns {x,y,w,h} in screen pixels for the node bounding box, or "{}" if not found.
     * @param {number} node_id
     * @returns {string}
     */
    node_screen_rect(node_id) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.dageditor_node_screen_rect(this.__wbg_ptr, node_id);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Called when a palette drag is cancelled (dropped outside canvas).
     */
    palette_cancel() {
        wasm.dageditor_palette_cancel(this.__wbg_ptr);
    }
    /**
     * Called on every mousemove while dragging from the palette.
     * @param {number} sx
     * @param {number} sy
     */
    palette_drag_move(sx, sy) {
        wasm.dageditor_palette_drag_move(this.__wbg_ptr, sx, sy);
    }
    /**
     * Called when the user starts dragging a node type from the HTML palette.
     * `kind_str` must match the serde snake_case name of a `NodeKind` variant.
     * @param {string} kind_str
     */
    palette_drag_start(kind_str) {
        const ptr0 = passStringToWasm0(kind_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.dageditor_palette_drag_start(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Called when the user releases a palette drag on the canvas.
     * @param {number} sx
     * @param {number} sy
     */
    palette_drop(sx, sy) {
        wasm.dageditor_palette_drop(this.__wbg_ptr, sx, sy);
    }
    /**
     * Returns JSON summary of available node kinds (palette + config schema).
     * @returns {string}
     */
    static palette_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.dageditor_palette_json();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    paste() {
        const ret = wasm.dageditor_paste(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    redo() {
        const ret = wasm.dageditor_redo(this.__wbg_ptr);
        return ret !== 0;
    }
    render() {
        wasm.dageditor_render(this.__wbg_ptr);
    }
    /**
     * Draw a result chart for node `node_id` onto a separate canvas element.
     * Call after `apply_results_json()`. Returns false if the node has no outputs.
     * @param {number} node_id
     * @param {string} chart_canvas_id
     * @returns {boolean}
     */
    render_chart(node_id, chart_canvas_id) {
        const ptr0 = passStringToWasm0(chart_canvas_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_render_chart(this.__wbg_ptr, node_id, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Render a scaled-down overview of the DAG onto a separate minimap canvas.
     * Draws node rects, edges, and a viewport indicator box.
     * @param {string} minimap_id
     * @param {number} main_canvas_w
     * @param {number} main_canvas_h
     * @returns {boolean}
     */
    render_minimap(minimap_id, main_canvas_w, main_canvas_h) {
        const ptr0 = passStringToWasm0(minimap_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_render_minimap(this.__wbg_ptr, ptr0, len0, main_canvas_w, main_canvas_h);
        return ret !== 0;
    }
    /**
     * Notify the editor that the canvas has been resized.
     */
    resize() {
        wasm.dageditor_resize(this.__wbg_ptr);
    }
    select_all_nodes() {
        wasm.dageditor_select_all_nodes(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    selected_node_count() {
        const ret = wasm.dageditor_selected_node_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the u32 ID of the selected node, or u32::MAX if nothing selected.
     * @returns {number}
     */
    selected_node_id() {
        const ret = wasm.dageditor_selected_node_id(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Update the config of node `node_id` with a JSON object.  Returns true on success.
     * @param {number} node_id
     * @param {string} config_json
     * @returns {boolean}
     */
    set_node_config(node_id, config_json) {
        const ptr0 = passStringToWasm0(config_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_set_node_config(this.__wbg_ptr, node_id, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Set a custom display label on a node (empty string clears it).
     * @param {number} node_id
     * @param {string} label
     * @returns {boolean}
     */
    set_node_label(node_id, label) {
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.dageditor_set_node_label(this.__wbg_ptr, node_id, ptr0, len0);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    snap_enabled() {
        const ret = wasm.dageditor_snap_enabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * JSON array of available template descriptors for the UI dropdown.
     * @returns {string}
     */
    static template_catalog_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.dageditor_template_catalog_json();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    to_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.dageditor_to_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Toggle snap-to-grid.  `grid_size` = world-space grid size in pixels (e.g. 20).
     * @param {number} grid_size
     * @returns {boolean}
     */
    toggle_snap(grid_size) {
        const ret = wasm.dageditor_toggle_snap(this.__wbg_ptr, grid_size);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    undo() {
        const ret = wasm.dageditor_undo(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {number} delta_y
     * @param {number} sx
     * @param {number} sy
     */
    wheel(delta_y, sx, sy) {
        wasm.dageditor_wheel(this.__wbg_ptr, delta_y, sx, sy);
    }
    /**
     * Fit all nodes into view with padding.
     */
    zoom_fit() {
        wasm.dageditor_zoom_fit(this.__wbg_ptr);
    }
    zoom_in() {
        wasm.dageditor_zoom_in(this.__wbg_ptr);
    }
    zoom_out() {
        wasm.dageditor_zoom_out(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    zoom_percent() {
        const ret = wasm.dageditor_zoom_percent(this.__wbg_ptr);
        return ret >>> 0;
    }
    zoom_reset() {
        wasm.dageditor_zoom_reset(this.__wbg_ptr);
    }
    /**
     * Fit view to selected nodes only; falls back to zoom_fit if nothing selected.
     */
    zoom_to_selected() {
        wasm.dageditor_zoom_to_selected(this.__wbg_ptr);
    }
}
if (Symbol.dispose) DagEditor.prototype[Symbol.dispose] = DagEditor.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_undefined_721f8decd50c87a3: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_ea4887a5f8f9a9db: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_arc_3ea3aa52ccc546bb: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.arc(arg1, arg2, arg3, arg4, arg5, arg6 !== 0);
        }, arguments); },
        __wbg_arc_74cf0c033e9df542: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_beginPath_c99b5be3516a2077: function(arg0) {
            arg0.beginPath();
        },
        __wbg_bezierCurveTo_22132b66df298a0b: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.bezierCurveTo(arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_clip_8ac1823db730edf8: function(arg0) {
            arg0.clip();
        },
        __wbg_closePath_47136fd7a8a2f043: function(arg0) {
            arg0.closePath();
        },
        __wbg_document_2634180a4c694068: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_fillRect_3c420f5077df8d3b: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_cdea0ac33ff3d2d1: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fill_b39141050e50c461: function(arg0) {
            arg0.fill();
        },
        __wbg_getContext_486aab500e1c34c9: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getElementById_c7aba6b93b34bf01: function(arg0, arg1, arg2) {
            const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_height_a04613570d793df2: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_instanceof_CanvasRenderingContext2d_d0cab9e931424c52: function(arg0) {
            let result;
            try {
                result = arg0 instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_8ce29a370a2b10a4: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_0d356b88a2f77c42: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_lineTo_2a649fce185f0bf0: function(arg0, arg1, arg2) {
            arg0.lineTo(arg1, arg2);
        },
        __wbg_moveTo_8973531c3399ba16: function(arg0, arg1, arg2) {
            arg0.moveTo(arg1, arg2);
        },
        __wbg_new_36e147a8ced3c6e0: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_push_f724b5db8acf89d2: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_rect_ec6fe62084b85fe8: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.rect(arg1, arg2, arg3, arg4);
        },
        __wbg_restore_6d0b3ce5b0ed7f95: function(arg0) {
            arg0.restore();
        },
        __wbg_save_38619d761125d8ce: function(arg0) {
            arg0.save();
        },
        __wbg_setLineDash_7394cefd476e675f: function() { return handleError(function (arg0, arg1) {
            arg0.setLineDash(arg1);
        }, arguments); },
        __wbg_setTransform_49a6e126738858db: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setTransform(arg1, arg2, arg3, arg4, arg5, arg6);
        }, arguments); },
        __wbg_set_fillStyle_35471aa9a10a6686: function(arg0, arg1, arg2) {
            arg0.fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_e2bce6175ef42bc3: function(arg0, arg1, arg2) {
            arg0.font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_globalAlpha_60fedcc06aa9a61c: function(arg0, arg1) {
            arg0.globalAlpha = arg1;
        },
        __wbg_set_lineWidth_fef15cb5c15a6cdc: function(arg0, arg1) {
            arg0.lineWidth = arg1;
        },
        __wbg_set_shadowBlur_062c1f25276f0434: function(arg0, arg1) {
            arg0.shadowBlur = arg1;
        },
        __wbg_set_shadowColor_100ad7306fd5addf: function(arg0, arg1, arg2) {
            arg0.shadowColor = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_shadowOffsetX_61a6196c2ef7006c: function(arg0, arg1) {
            arg0.shadowOffsetX = arg1;
        },
        __wbg_set_shadowOffsetY_545ef89bd50db5bd: function(arg0, arg1) {
            arg0.shadowOffsetY = arg1;
        },
        __wbg_set_strokeStyle_d494db5851ff0dbd: function(arg0, arg1, arg2) {
            arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textAlign_8cc28de727b5df6f: function(arg0, arg1, arg2) {
            arg0.textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_textBaseline_6f233751bd79619e: function(arg0, arg1, arg2) {
            arg0.textBaseline = getStringFromWasm0(arg1, arg2);
        },
        __wbg_static_accessor_GLOBAL_THIS_2fee5048bcca5938: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_ce44e66a4935da8c: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_44f6e0cb5e67cdad: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_168f178805d978fe: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_strokeRect_a9cb57c3713e908d: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.strokeRect(arg1, arg2, arg3, arg4);
        },
        __wbg_stroke_d0c2cfbe28711bcb: function(arg0) {
            arg0.stroke();
        },
        __wbg_width_c8740d5bdf596189: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./hydrocomplete_dag_bg.js": import0,
    };
}

const DagEditorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dageditor_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('hydrocomplete_dag_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
