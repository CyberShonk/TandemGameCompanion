#include <windows.h>
#include <stdio.h>
#include <string.h>

#define TIMER_SHOW_DISABLED 1
#define TIMER_ENABLE_TARGET 2
#define TIMER_MONITOR 3
#define TIMER_CLOSE 4

#define MODE_IMPOSTOR 1
#define MODE_SCOPED 2
#define MODE_NOOP 3
#define MODE_OUT_OF_RANGE 4
#define MODE_AMBIGUOUS_PARENT 5
#define MODE_AMBIGUOUS_CONTROL 6
#define MODE_WRONG_RUNTIME_CLASS 7

static const char *event_path = NULL;
static int mode = 0;
static int target_control_id = 0;
static HWND target_window = NULL;
static HWND other_window = NULL;
static HWND target_combo = NULL;
static HWND wrong_class_control = NULL;
static HWND wrong_id_combo = NULL;
static HWND other_window_combo = NULL;
static DWORD disabled_delay_ms = 0;
static int notification_count = 0;
static int selection_event_written = 0;
static int stable_event_written = 0;
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
    char line[256];
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

static void record_failure(const char *event) {
    if (append_event(event) != 0) {
        event_failure = 1;
    }
}

static int selected_index(HWND combo) {
    if (combo == NULL || !IsWindow(combo)) {
        return -2;
    }
    return (int)SendMessageA(combo, CB_GETCURSEL, 0, 0);
}

static void monitor_controls(void) {
    if (mode == MODE_SCOPED) {
        if (wrong_id_combo != NULL && selected_index(wrong_id_combo) != 0) {
            record_failure("VIOLATION-combo-correct-class-wrong-id-mutated");
        }
        if (other_window_combo != NULL && selected_index(other_window_combo) != 0) {
            record_failure("VIOLATION-combo-other-window-mutated");
        }
        if (notification_count > 1) {
            record_failure("VIOLATION-combo-target-multiple-notifications");
        }
        if (!selection_event_written && selected_index(target_combo) == 2) {
            record_failure("combo-target-index-2");
            if (SendMessageA(wrong_class_control, BM_GETCHECK, 0, 0) == BST_UNCHECKED) {
                record_failure("combo-correct-id-wrong-class-unchanged");
            } else {
                record_failure("VIOLATION-combo-correct-id-wrong-class-mutated");
            }
            if (selected_index(wrong_id_combo) == 0) {
                record_failure("combo-correct-class-wrong-id-index-0");
            }
            if (selected_index(other_window_combo) == 0) {
                record_failure("combo-other-window-index-0");
            }
            selection_event_written = 1;
        }
    } else if (mode == MODE_IMPOSTOR) {
        if (selected_index(target_combo) != 0) {
            record_failure("VIOLATION-combo-other-process-mutated");
        }
        if (notification_count != 0) {
            record_failure("VIOLATION-combo-other-process-notified");
        }
    } else if (mode == MODE_NOOP) {
        if (selected_index(target_combo) != 2) {
            record_failure("VIOLATION-combo-noop-selection-changed");
        }
        if (notification_count != 0) {
            record_failure("VIOLATION-combo-noop-notified");
        }
    } else if (mode == MODE_OUT_OF_RANGE) {
        if (selected_index(target_combo) != 0) {
            record_failure("VIOLATION-combo-out-of-range-mutated");
        }
        if (notification_count != 0) {
            record_failure("VIOLATION-combo-out-of-range-notified");
        }
        if (!stable_event_written) {
            record_failure("combo-out-of-range-still-index-0");
            stable_event_written = 1;
        }
    } else if (mode == MODE_WRONG_RUNTIME_CLASS) {
        if (wrong_class_control != NULL
            && SendMessageA(wrong_class_control, BM_GETCHECK, 0, 0) != BST_UNCHECKED) {
            record_failure("VIOLATION-combo-wrong-runtime-class-mutated");
        }
        if (notification_count != 0) {
            record_failure("VIOLATION-combo-wrong-runtime-class-notified");
        }
    }
}

static LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
        case WM_COMMAND:
            if (HIWORD(wparam) == CBN_SELCHANGE) {
                HWND source = (HWND)lparam;
                int control_id = LOWORD(wparam);
                if (window == target_window
                    && source == target_combo
                    && control_id == target_control_id) {
                    notification_count++;
                    if (mode == MODE_SCOPED
                        && notification_count == 1
                        && selected_index(target_combo) == 2) {
                        record_failure("combo-target-notification-index-2");
                    } else if (mode == MODE_SCOPED) {
                        record_failure("VIOLATION-combo-target-notification-invalid");
                    } else if (mode == MODE_NOOP) {
                        record_failure("VIOLATION-combo-noop-notified");
                    } else if (mode == MODE_OUT_OF_RANGE) {
                        record_failure("VIOLATION-combo-out-of-range-notified");
                    } else if (mode == MODE_IMPOSTOR) {
                        record_failure("VIOLATION-combo-other-process-notified");
                    }
                } else {
                    record_failure("VIOLATION-combo-unexpected-selection-notification");
                }
                return 0;
            }
            break;

        case WM_TIMER:
            if (window != target_window) {
                return 0;
            }

            if (wparam == TIMER_SHOW_DISABLED && mode == MODE_SCOPED) {
                KillTimer(window, TIMER_SHOW_DISABLED);
                EnableWindow(target_combo, FALSE);
                ShowWindow(target_combo, SW_SHOW);
                UpdateWindow(target_combo);
                if (selected_index(target_combo) != 0) {
                    record_failure("VIOLATION-combo-selected-before-visible-disabled-stage");
                }
                record_failure("combo-target-visible-disabled-index-0");
                if (SetTimer(window, TIMER_ENABLE_TARGET, disabled_delay_ms, NULL) == 0) {
                    event_failure = 1;
                    DestroyWindow(window);
                }
                return 0;
            }

            if (wparam == TIMER_ENABLE_TARGET && mode == MODE_SCOPED) {
                KillTimer(window, TIMER_ENABLE_TARGET);
                if (selected_index(target_combo) != 0) {
                    record_failure("VIOLATION-combo-selected-before-enabled-stage");
                }
                EnableWindow(target_combo, TRUE);
                record_failure("combo-target-visible-enabled-index-0");
                return 0;
            }

            if (wparam == TIMER_MONITOR) {
                monitor_controls();
                return 0;
            }

            if (wparam == TIMER_CLOSE) {
                KillTimer(window, TIMER_CLOSE);
                monitor_controls();
                if (mode == MODE_IMPOSTOR) {
                    record_failure("combo-other-process-final-index-0");
                } else if (mode == MODE_NOOP) {
                    record_failure("combo-noop-final-index-2-no-notification");
                } else if (mode == MODE_SCOPED) {
                    if (selected_index(wrong_id_combo) == 0) {
                        record_failure("combo-correct-class-wrong-id-final-index-0");
                    }
                    if (selected_index(other_window_combo) == 0) {
                        record_failure("combo-other-window-final-index-0");
                    }
                }
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

static HWND create_top_level(HINSTANCE instance, const char *class_name, const char *title) {
    return CreateWindowExA(
        0,
        class_name,
        title,
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        560,
        320,
        NULL,
        NULL,
        instance,
        NULL
    );
}

static HWND create_control(
    HWND parent,
    const char *class_name,
    DWORD style,
    int control_id,
    int x,
    int y
) {
    return CreateWindowExA(
        0,
        class_name,
        "",
        WS_CHILD | style,
        x,
        y,
        240,
        120,
        parent,
        (HMENU)(INT_PTR)control_id,
        GetModuleHandleA(NULL),
        NULL
    );
}

static HWND create_combo(HWND parent, DWORD style, int control_id, int x, int y, int initial) {
    HWND combo = create_control(
        parent,
        "ComboBox",
        style | WS_VSCROLL | CBS_DROPDOWNLIST,
        control_id,
        x,
        y
    );
    if (combo == NULL) {
        return NULL;
    }

    if (SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"Zero") < 0
        || SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"One") < 0
        || SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"Two") < 0
        || SendMessageA(combo, CB_SETCURSEL, (WPARAM)initial, 0) != initial) {
        DestroyWindow(combo);
        return NULL;
    }

    return combo;
}

static int start_common_timers(DWORD lifetime_ms) {
    if (SetTimer(target_window, TIMER_MONITOR, 25, NULL) == 0
        || SetTimer(target_window, TIMER_CLOSE, lifetime_ms, NULL) == 0) {
        return 1;
    }
    return 0;
}

static int run_impostor(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    record_failure("combo-impostor-start");
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    target_combo = create_combo(
        target_window,
        WS_VISIBLE,
        target_control_id,
        24,
        24,
        0
    );
    if (target_combo == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_failure("combo-impostor-ready-index-0");
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
    record_failure("combo-scoped-start");
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }

    wrong_class_control = create_control(
        target_window,
        "Button",
        WS_VISIBLE | BS_PUSHBUTTON,
        target_control_id,
        24,
        24
    );
    wrong_id_combo = create_combo(
        target_window,
        WS_VISIBLE,
        target_control_id + 1,
        24,
        72,
        0
    );
    target_combo = create_combo(
        target_window,
        0,
        target_control_id,
        24,
        120,
        0
    );
    if (wrong_class_control == NULL || wrong_id_combo == NULL || target_combo == NULL) {
        return 5;
    }

    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_failure("combo-selector-decoys-ready");
    if (selected_index(target_combo) != 0) {
        record_failure("VIOLATION-combo-target-not-zero-while-hidden");
    }
    record_failure("combo-target-hidden-index-0");

    other_window = create_top_level(
        instance,
        class_name,
        "Different Combo Trainer Top-Level Window"
    );
    if (other_window == NULL) {
        return 6;
    }
    other_window_combo = create_combo(
        other_window,
        WS_VISIBLE,
        target_control_id,
        24,
        24,
        0
    );
    if (other_window_combo == NULL) {
        return 7;
    }
    ShowWindow(other_window, SW_SHOW);
    UpdateWindow(other_window);
    record_failure("combo-other-window-ready-index-0");

    disabled_delay_ms = disabled_ms;
    if (SetTimer(target_window, TIMER_SHOW_DISABLED, hidden_delay_ms, NULL) == 0
        || start_common_timers(lifetime_ms) != 0) {
        return 8;
    }
    return 0;
}

static int run_noop(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    target_combo = create_combo(
        target_window,
        WS_VISIBLE,
        target_control_id,
        24,
        24,
        2
    );
    if (target_combo == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_failure("combo-noop-ready-index-2");
    return start_common_timers(lifetime_ms) == 0 ? 0 : 6;
}

static int run_out_of_range(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    DWORD lifetime_ms
) {
    target_window = create_top_level(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }
    target_combo = create_combo(
        target_window,
        WS_VISIBLE,
        target_control_id,
        24,
        24,
        0
    );
    if (target_combo == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_failure("combo-out-of-range-ready-index-0");
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
    wrong_class_control = create_control(
        target_window,
        "Button",
        WS_VISIBLE | BS_AUTOCHECKBOX,
        target_control_id,
        24,
        24
    );
    if (wrong_class_control == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_failure("combo-wrong-runtime-class-ready");
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
    target_combo = create_combo(target_window, WS_VISIBLE, target_control_id, 24, 24, 0);
    other_window_combo = create_combo(other_window, WS_VISIBLE, target_control_id, 24, 24, 0);
    if (target_combo == NULL || other_window_combo == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    ShowWindow(other_window, SW_SHOW);
    UpdateWindow(target_window);
    UpdateWindow(other_window);
    record_failure("combo-ambiguous-parent-ready");
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
    target_combo = create_combo(target_window, WS_VISIBLE, target_control_id, 24, 24, 0);
    wrong_id_combo = create_combo(target_window, WS_VISIBLE, target_control_id, 24, 104, 0);
    if (target_combo == NULL || wrong_id_combo == NULL) {
        return 5;
    }
    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);
    record_failure("combo-ambiguous-control-ready");
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
    } else if (strcmp(mode_name, "noop") == 0) {
        mode = MODE_NOOP;
    } else if (strcmp(mode_name, "out-of-range") == 0) {
        mode = MODE_OUT_OF_RANGE;
    } else if (strcmp(mode_name, "ambiguous-parent") == 0) {
        mode = MODE_AMBIGUOUS_PARENT;
    } else if (strcmp(mode_name, "ambiguous-control") == 0) {
        mode = MODE_AMBIGUOUS_CONTROL;
    } else if (strcmp(mode_name, "wrong-runtime-class") == 0) {
        mode = MODE_WRONG_RUNTIME_CLASS;
    } else {
        return 2;
    }

    HINSTANCE instance = GetModuleHandleA(NULL);
    WNDCLASSA window_class = {0};
    window_class.lpfnWndProc = window_proc;
    window_class.hInstance = instance;
    window_class.lpszClassName = "TandemComboSmokeHelper";
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
    } else if (mode == MODE_NOOP) {
        result = run_noop(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_OUT_OF_RANGE) {
        result = run_out_of_range(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_AMBIGUOUS_PARENT) {
        result = run_ambiguous_parent(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_AMBIGUOUS_CONTROL) {
        result = run_ambiguous_control(instance, window_class.lpszClassName, title, lifetime_ms);
    } else if (mode == MODE_WRONG_RUNTIME_CLASS) {
        result = run_wrong_runtime_class(
            instance,
            window_class.lpszClassName,
            title,
            lifetime_ms
        );
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
