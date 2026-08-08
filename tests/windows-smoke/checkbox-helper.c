#include <windows.h>
#include <stdio.h>
#include <string.h>

#define TIMER_SHOW_DISABLED 1
#define TIMER_ENABLE_TARGET 2
#define TIMER_MONITOR 3
#define TIMER_CLOSE 4

#define MODE_IMPOSTOR 1
#define MODE_SCOPED 2
#define MODE_NOOP_CHECKED 3
#define MODE_UNCHECK 4
#define MODE_HIDDEN 5
#define MODE_DISABLED 6
#define MODE_MANUAL 7
#define MODE_THREESTATE 8
#define MODE_RADIO 9
#define MODE_OWNERDRAW 10
#define MODE_WRONG_RUNTIME_CLASS 11
#define MODE_AMBIGUOUS_PARENT 12
#define MODE_AMBIGUOUS_CONTROL 13

static const char *event_path = NULL;
static int mode = 0;
static int target_control_id = 0;
static HWND target_window = NULL;
static HWND other_window = NULL;
static HWND target_checkbox = NULL;
static HWND wrong_id_checkbox = NULL;
static HWND other_window_checkbox = NULL;
static DWORD disabled_delay_ms = 0;
static int target_click_count = 0;
static int wrong_id_click_count = 0;
static int other_window_click_count = 0;
static int event_failure = 0;

static DWORD parse_unsigned(const char *text) {
    DWORD value = 0;
    while (*text >= '0' && *text <= '9') {
        value = (value * 10) + (DWORD)(*text - '0');
        text++;
    }
    return value;
}

static int append_event(const char *event) {
    char line[320];
    int length = snprintf(line, sizeof(line), "%s\r\n", event);
    if (length < 0 || length >= (int)sizeof(line)) {
        return 1;
    }
    HANDLE file = CreateFileA(
        event_path,
        FILE_APPEND_DATA,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        NULL,
        OPEN_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );
    if (file == INVALID_HANDLE_VALUE) {
        return 1;
    }
    DWORD written = 0;
    BOOL ok = WriteFile(file, line, (DWORD)length, &written, NULL);
    CloseHandle(file);
    return ok && written == (DWORD)length ? 0 : 1;
}

static void record_event(const char *event) {
    if (append_event(event) != 0) {
        event_failure = 1;
    }
}

static void record_violation(const char *event) {
    record_event(event);
}

static int checked_state(HWND control) {
    LRESULT value = SendMessageA(control, BM_GETCHECK, 0, 0);
    if (value == BST_UNCHECKED) {
        return 0;
    }
    if (value == BST_CHECKED) {
        return 1;
    }
    return 2;
}

static void set_initial_checked(HWND control, int checked) {
    SendMessageA(control, BM_SETCHECK, checked ? BST_CHECKED : BST_UNCHECKED, 0);
}

static HWND create_top_level(HINSTANCE instance, const char *class_name, const char *title) {
    return CreateWindowExA(
        0,
        class_name,
        title,
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        640,
        360,
        NULL,
        NULL,
        instance,
        NULL
    );
}

static HWND create_control(
    HWND parent,
    const char *class_name,
    const char *text,
    DWORD style,
    int control_id,
    int x,
    int y
) {
    return CreateWindowExA(
        0,
        class_name,
        text,
        WS_CHILD | style,
        x,
        y,
        220,
        34,
        parent,
        (HMENU)(INT_PTR)control_id,
        GetModuleHandleA(NULL),
        NULL
    );
}

static HWND create_checkbox(
    HWND parent,
    const char *text,
    DWORD style,
    int control_id,
    int x,
    int y,
    int initially_checked
) {
    HWND control = create_control(parent, "Button", text, style, control_id, x, y);
    if (control != NULL) {
        set_initial_checked(control, initially_checked);
    }
    return control;
}

static int start_common_timers(DWORD lifetime_ms) {
    if (SetTimer(target_window, TIMER_MONITOR, 25, NULL) == 0
        || SetTimer(target_window, TIMER_CLOSE, lifetime_ms, NULL) == 0) {
        return 1;
    }
    return 0;
}

static void schedule_settle_close(HWND window) {
    KillTimer(window, TIMER_CLOSE);
    if (SetTimer(window, TIMER_CLOSE, 400, NULL) == 0) {
        event_failure = 1;
        DestroyWindow(window);
    }
}

static void monitor_state(void) {
    if (target_checkbox != NULL && IsWindow(target_checkbox)) {
        int state = checked_state(target_checkbox);
        if (target_click_count > 1) {
            record_violation("VIOLATION-checkbox-target-clicked-more-than-once");
        }
        if (mode == MODE_IMPOSTOR && (target_click_count != 0 || state != 0)) {
            record_violation("VIOLATION-checkbox-other-process-mutated");
        }
        if (mode == MODE_SCOPED) {
            if (target_click_count == 0 && state != 0) {
                record_violation("VIOLATION-checkbox-target-changed-before-click");
            }
            if (target_click_count == 1 && state != 1) {
                record_violation("VIOLATION-checkbox-target-click-did-not-check");
            }
        }
        if (mode == MODE_NOOP_CHECKED && (target_click_count != 0 || state != 1)) {
            record_violation("VIOLATION-checkbox-noop-checked-mutated");
        }
        if (mode == MODE_UNCHECK) {
            if (target_click_count == 0 && state != 1) {
                record_violation("VIOLATION-checkbox-uncheck-changed-before-click");
            }
            if (target_click_count == 1 && state != 0) {
                record_violation("VIOLATION-checkbox-uncheck-click-did-not-clear");
            }
        }
        if ((mode == MODE_HIDDEN
             || mode == MODE_DISABLED
             || mode == MODE_MANUAL
             || mode == MODE_THREESTATE
             || mode == MODE_RADIO
             || mode == MODE_OWNERDRAW)
            && (target_click_count != 0 || state != 0)) {
            record_violation("VIOLATION-checkbox-rejected-control-mutated");
        }
    }

    if (wrong_id_checkbox != NULL && IsWindow(wrong_id_checkbox)) {
        if (wrong_id_click_count != 0 || checked_state(wrong_id_checkbox) != 0) {
            record_violation("VIOLATION-checkbox-correct-class-wrong-id-mutated");
        }
    }
    if (other_window_checkbox != NULL && IsWindow(other_window_checkbox)) {
        if (other_window_click_count != 0 || checked_state(other_window_checkbox) != 0) {
            record_violation("VIOLATION-checkbox-other-window-mutated");
        }
    }
}

static void record_final_state(void) {
    monitor_state();
    if (mode == MODE_IMPOSTOR
        && checked_state(target_checkbox) == 0
        && target_click_count == 0) {
        record_event("checkbox-other-process-final-checked-0-click-count-0");
    } else if (mode == MODE_SCOPED) {
        if (checked_state(target_checkbox) == 1 && target_click_count == 1) {
            record_event("checkbox-target-final-checked-1-click-count-1");
        }
        if (checked_state(wrong_id_checkbox) == 0 && wrong_id_click_count == 0) {
            record_event("checkbox-correct-class-wrong-id-final-checked-0-click-count-0");
        }
        if (checked_state(other_window_checkbox) == 0 && other_window_click_count == 0) {
            record_event("checkbox-other-window-final-checked-0-click-count-0");
        }
    } else if (mode == MODE_NOOP_CHECKED
               && checked_state(target_checkbox) == 1
               && target_click_count == 0) {
        record_event("checkbox-noop-final-checked-1-click-count-0");
    } else if (mode == MODE_UNCHECK
               && checked_state(target_checkbox) == 0
               && target_click_count == 1) {
        record_event("checkbox-uncheck-final-checked-0-click-count-1");
    }
}

static LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
        case WM_COMMAND:
            if (HIWORD(wparam) == BN_CLICKED) {
                HWND source = (HWND)lparam;
                int control_id = LOWORD(wparam);

                if (window == target_window
                    && source == target_checkbox
                    && control_id == target_control_id) {
                    target_click_count++;
                    if (mode == MODE_SCOPED && target_click_count == 1) {
                        if (checked_state(target_checkbox) == 1) {
                            record_event("checkbox-target-click-1-checked-1");
                        } else {
                            record_violation("VIOLATION-checkbox-target-click-not-checked");
                        }
                        schedule_settle_close(window);
                    } else if (mode == MODE_UNCHECK && target_click_count == 1) {
                        if (checked_state(target_checkbox) == 0) {
                            record_event("checkbox-uncheck-click-1-checked-0");
                        } else {
                            record_violation("VIOLATION-checkbox-uncheck-click-not-cleared");
                        }
                        schedule_settle_close(window);
                    } else {
                        record_violation("VIOLATION-checkbox-target-unexpected-click");
                    }
                    return 0;
                }

                if (window == target_window
                    && source == wrong_id_checkbox
                    && control_id == target_control_id + 1) {
                    wrong_id_click_count++;
                    record_violation("VIOLATION-checkbox-correct-class-wrong-id-clicked");
                    return 0;
                }

                if (window == other_window
                    && source == other_window_checkbox
                    && control_id == target_control_id) {
                    other_window_click_count++;
                    record_violation("VIOLATION-checkbox-other-window-clicked");
                    return 0;
                }

                record_violation("VIOLATION-checkbox-unexpected-command-notification");
                return 0;
            }
            break;

        case WM_TIMER:
            if (window != target_window) {
                return 0;
            }

            if (wparam == TIMER_SHOW_DISABLED && mode == MODE_SCOPED) {
                KillTimer(window, TIMER_SHOW_DISABLED);
                EnableWindow(target_checkbox, FALSE);
                ShowWindow(target_checkbox, SW_SHOW);
                UpdateWindow(target_checkbox);
                if (checked_state(target_checkbox) != 0 || target_click_count != 0) {
                    record_violation("VIOLATION-checkbox-mutated-before-visible-disabled-stage");
                }
                record_event("checkbox-target-visible-disabled-checked-0-click-count-0");
                if (SetTimer(window, TIMER_ENABLE_TARGET, disabled_delay_ms, NULL) == 0) {
                    event_failure = 1;
                    DestroyWindow(window);
                }
                return 0;
            }

            if (wparam == TIMER_ENABLE_TARGET && mode == MODE_SCOPED) {
                KillTimer(window, TIMER_ENABLE_TARGET);
                if (checked_state(target_checkbox) != 0 || target_click_count != 0) {
                    record_violation("VIOLATION-checkbox-mutated-before-enabled-stage");
                }
                EnableWindow(target_checkbox, TRUE);
                record_event("checkbox-target-visible-enabled-checked-0-click-count-0");
                return 0;
            }

            if (wparam == TIMER_MONITOR) {
                monitor_state();
                return 0;
            }

            if (wparam == TIMER_CLOSE) {
                KillTimer(window, TIMER_CLOSE);
                record_final_state();
                DestroyWindow(window);
                return 0;
            }
            break;

        case WM_CLOSE:
            DestroyWindow(window);
            return 0;

        case WM_DESTROY:
            if (window == target_window) {
                if (other_window != NULL && IsWindow(other_window)) {
                    DestroyWindow(other_window);
                }
                PostQuitMessage(0);
            }
            return 0;

        default:
            break;
    }
    return DefWindowProcA(window, message, wparam, lparam);
}

static int run_impostor(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    record_event("checkbox-impostor-start");
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    target_checkbox = create_checkbox(
        target_window,
        "Enable",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        24,
        0
    );
    if (target_checkbox == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_event("checkbox-impostor-ready-checked-0-click-count-0");
    return start_common_timers(lifetime_ms) == 0 ? 0 : 6;
}

static int run_scoped(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD hidden_delay_ms,
    DWORD disabled_ms,
    DWORD lifetime_ms
) {
    record_event("checkbox-scoped-start");
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }

    HWND wrong_class_control = create_control(
        target_window,
        "Static",
        "Wrong runtime class",
        WS_VISIBLE | SS_LEFT,
        target_control_id,
        24,
        24
    );
    wrong_id_checkbox = create_checkbox(
        target_window,
        "Wrong ID",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id + 1,
        24,
        68,
        0
    );
    HWND manual_decoy = create_checkbox(
        target_window,
        "Manual checkbox",
        WS_VISIBLE | BS_CHECKBOX,
        target_control_id + 2,
        260,
        24,
        0
    );
    HWND radio_decoy = create_checkbox(
        target_window,
        "Radio decoy",
        WS_VISIBLE | BS_AUTORADIOBUTTON,
        target_control_id + 3,
        260,
        68,
        0
    );
    HWND three_state_decoy = create_checkbox(
        target_window,
        "Three-state decoy",
        WS_VISIBLE | BS_AUTO3STATE,
        target_control_id + 4,
        260,
        112,
        0
    );
    HWND ownerdraw_decoy = create_checkbox(
        target_window,
        "Owner-draw decoy",
        WS_VISIBLE | BS_OWNERDRAW,
        target_control_id + 5,
        260,
        156,
        0
    );
    target_checkbox = create_checkbox(
        target_window,
        "Enable feature",
        BS_AUTOCHECKBOX,
        target_control_id,
        24,
        112,
        0
    );

    if (wrong_class_control == NULL
        || wrong_id_checkbox == NULL
        || manual_decoy == NULL
        || radio_decoy == NULL
        || three_state_decoy == NULL
        || ownerdraw_decoy == NULL
        || target_checkbox == NULL) {
        return 5;
    }

    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_event("checkbox-selector-decoys-ready");
    record_event("checkbox-target-hidden-checked-0-click-count-0");

    other_window = create_top_level(instance, class_name, "Different Checkbox Trainer Window");
    if (other_window == NULL) {
        return 6;
    }
    other_window_checkbox = create_checkbox(
        other_window,
        "Enable feature",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        24,
        0
    );
    if (other_window_checkbox == NULL) {
        return 7;
    }
    ShowWindow(other_window, SW_SHOW);
    UpdateWindow(other_window);
    record_event("checkbox-other-window-ready-checked-0-click-count-0");

    disabled_delay_ms = disabled_ms;
    if (SetTimer(target_window, TIMER_SHOW_DISABLED, hidden_delay_ms, NULL) == 0
        || start_common_timers(lifetime_ms) != 0) {
        return 8;
    }
    return 0;
}

static int run_single_checkbox(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD style,
    BOOL visible,
    BOOL enabled,
    int initially_checked,
    const char *ready_event,
    DWORD lifetime_ms
) {
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    target_checkbox = create_checkbox(
        target_window,
        "Enable feature",
        (visible ? WS_VISIBLE : 0) | style,
        target_control_id,
        24,
        24,
        initially_checked
    );
    if (target_checkbox == NULL) {
        return 5;
    }
    EnableWindow(target_checkbox, enabled);
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_event(ready_event);
    return start_common_timers(lifetime_ms) == 0 ? 0 : 6;
}

static int run_wrong_runtime_class(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    HWND control = create_control(
        target_window,
        "Static",
        "Wrong runtime class",
        WS_VISIBLE | SS_LEFT,
        target_control_id,
        24,
        24
    );
    if (control == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_event("checkbox-wrong-runtime-class-ready");
    return start_common_timers(lifetime_ms) == 0 ? 0 : 6;
}

static int run_ambiguous_parent(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    target_window = create_top_level(instance, class_name, title);
    other_window = create_top_level(instance, class_name, title);
    if (target_window == NULL || other_window == NULL) {
        return 4;
    }
    target_checkbox = create_checkbox(
        target_window,
        "Enable one",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        24,
        0
    );
    other_window_checkbox = create_checkbox(
        other_window,
        "Enable two",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        24,
        0
    );
    if (target_checkbox == NULL || other_window_checkbox == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    ShowWindow(other_window, SW_SHOW);
    UpdateWindow(target_window);
    UpdateWindow(other_window);
    record_event("checkbox-ambiguous-parent-ready");
    return start_common_timers(lifetime_ms) == 0 ? 0 : 6;
}

static int run_ambiguous_control(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    target_checkbox = create_checkbox(
        target_window,
        "Enable one",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        24,
        0
    );
    wrong_id_checkbox = create_checkbox(
        target_window,
        "Enable two",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        68,
        0
    );
    if (target_checkbox == NULL || wrong_id_checkbox == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_event("checkbox-ambiguous-control-ready");
    return start_common_timers(lifetime_ms) == 0 ? 0 : 6;
}

int main(int argc, char **argv) {
    if (argc != 8) {
        return 2;
    }

    event_path = argv[1];
    const char *mode_name = argv[2];
    const char *title = argv[3];
    target_control_id = (int)parse_unsigned(argv[4]);
    DWORD delay_one_ms = parse_unsigned(argv[5]);
    DWORD delay_two_ms = parse_unsigned(argv[6]);
    DWORD lifetime_ms = parse_unsigned(argv[7]);

    if (strcmp(mode_name, "impostor") == 0) {
        mode = MODE_IMPOSTOR;
    } else if (strcmp(mode_name, "scoped") == 0) {
        mode = MODE_SCOPED;
    } else if (strcmp(mode_name, "noop-checked") == 0) {
        mode = MODE_NOOP_CHECKED;
    } else if (strcmp(mode_name, "uncheck") == 0) {
        mode = MODE_UNCHECK;
    } else if (strcmp(mode_name, "hidden") == 0) {
        mode = MODE_HIDDEN;
    } else if (strcmp(mode_name, "disabled") == 0) {
        mode = MODE_DISABLED;
    } else if (strcmp(mode_name, "manual") == 0) {
        mode = MODE_MANUAL;
    } else if (strcmp(mode_name, "threestate") == 0) {
        mode = MODE_THREESTATE;
    } else if (strcmp(mode_name, "radio") == 0) {
        mode = MODE_RADIO;
    } else if (strcmp(mode_name, "ownerdraw") == 0) {
        mode = MODE_OWNERDRAW;
    } else if (strcmp(mode_name, "wrong-runtime-class") == 0) {
        mode = MODE_WRONG_RUNTIME_CLASS;
    } else if (strcmp(mode_name, "ambiguous-parent") == 0) {
        mode = MODE_AMBIGUOUS_PARENT;
    } else if (strcmp(mode_name, "ambiguous-control") == 0) {
        mode = MODE_AMBIGUOUS_CONTROL;
    } else {
        return 2;
    }

    HINSTANCE instance = GetModuleHandleA(NULL);
    WNDCLASSA window_class = {0};
    window_class.lpfnWndProc = window_proc;
    window_class.hInstance = instance;
    window_class.lpszClassName = "TandemCheckboxSmokeHelper";
    if (!RegisterClassA(&window_class) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
        return 3;
    }

    int result = 0;
    if (mode == MODE_IMPOSTOR) {
        result = run_impostor(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_SCOPED) {
        result = run_scoped(
            instance,
            window_class.lpszClassName,
            title,
            delay_one_ms,
            delay_two_ms,
            lifetime_ms
        );
    } else if (mode == MODE_NOOP_CHECKED) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_AUTOCHECKBOX,
            TRUE,
            TRUE,
            1,
            "checkbox-noop-ready-checked-1-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_UNCHECK) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_AUTOCHECKBOX,
            TRUE,
            TRUE,
            1,
            "checkbox-uncheck-ready-checked-1-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_HIDDEN) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_AUTOCHECKBOX,
            FALSE,
            TRUE,
            0,
            "checkbox-hidden-ready-checked-0-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_DISABLED) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_AUTOCHECKBOX,
            TRUE,
            FALSE,
            0,
            "checkbox-disabled-ready-checked-0-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_MANUAL) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_CHECKBOX,
            TRUE,
            TRUE,
            0,
            "checkbox-manual-ready-checked-0-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_THREESTATE) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_AUTO3STATE,
            TRUE,
            TRUE,
            0,
            "checkbox-threestate-ready-checked-0-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_RADIO) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_AUTORADIOBUTTON,
            TRUE,
            TRUE,
            0,
            "checkbox-radio-ready-checked-0-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_OWNERDRAW) {
        result = run_single_checkbox(
            instance,
            window_class.lpszClassName,
            title,
            BS_OWNERDRAW,
            TRUE,
            TRUE,
            0,
            "checkbox-ownerdraw-ready-checked-0-click-count-0",
            lifetime_ms
        );
    } else if (mode == MODE_WRONG_RUNTIME_CLASS) {
        result = run_wrong_runtime_class(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_AMBIGUOUS_PARENT) {
        result = run_ambiguous_parent(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_AMBIGUOUS_CONTROL) {
        result = run_ambiguous_control(instance, window_class.lpszClassName, title, lifetime_ms);
    }

    if (result != 0) {
        if (target_window != NULL && IsWindow(target_window)) {
            DestroyWindow(target_window);
        }
        return result;
    }

    MSG message;
    while (GetMessageA(&message, NULL, 0, 0) > 0) {
        TranslateMessage(&message);
        DispatchMessageA(&message);
    }

    return event_failure ? 12 : 0;
}
