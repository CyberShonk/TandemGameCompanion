#include <windows.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

#define TIMER_SHOW_DISABLED 1
#define TIMER_ENABLE_TARGET 2
#define TIMER_MONITOR 3
#define TIMER_CLOSE 4

#define MODE_IMPOSTOR 1
#define MODE_SCOPED 2
#define MODE_NOOP 3
#define MODE_UNICODE 4
#define MODE_CLEAR 5
#define MODE_REDACTION 6
#define MODE_HIDDEN 7
#define MODE_DISABLED 8
#define MODE_MULTILINE 9
#define MODE_PASSWORD 10
#define MODE_READONLY 11
#define MODE_UPPERCASE 12
#define MODE_LOWERCASE 13
#define MODE_OEMCONVERT 14
#define MODE_WRONG_RUNTIME_CLASS 15
#define MODE_AMBIGUOUS_PARENT 16
#define MODE_AMBIGUOUS_CONTROL 17
#define MODE_EXIT 18

static const char *event_path = NULL;
static int mode = 0;
static int target_control_id = 0;
static HWND target_window = NULL;
static HWND other_window = NULL;
static HWND target_edit = NULL;
static HWND wrong_id_edit = NULL;
static HWND other_window_edit = NULL;
static HWND duplicate_edit = NULL;
static DWORD show_delay_ms = 0;
static DWORD enable_delay_ms = 0;
static DWORD lifetime_ms = 0;
static int target_update_count = 0;
static int target_change_count = 0;
static int wrong_id_update_count = 0;
static int wrong_id_change_count = 0;
static int other_window_update_count = 0;
static int other_window_change_count = 0;
static int event_failure = 0;
static int suppress_notifications = 0;

static DWORD parse_unsigned(const char *text) {
    DWORD value = 0;
    while (*text >= '0' && *text <= '9') {
        value = (value * 10) + (DWORD)(*text - '0');
        text++;
    }
    return value;
}

static int parse_mode(const char *text) {
    if (strcmp(text, "impostor") == 0) return MODE_IMPOSTOR;
    if (strcmp(text, "scoped") == 0) return MODE_SCOPED;
    if (strcmp(text, "noop") == 0) return MODE_NOOP;
    if (strcmp(text, "unicode") == 0) return MODE_UNICODE;
    if (strcmp(text, "clear") == 0) return MODE_CLEAR;
    if (strcmp(text, "redaction") == 0) return MODE_REDACTION;
    if (strcmp(text, "hidden") == 0) return MODE_HIDDEN;
    if (strcmp(text, "disabled") == 0) return MODE_DISABLED;
    if (strcmp(text, "multiline") == 0) return MODE_MULTILINE;
    if (strcmp(text, "password") == 0) return MODE_PASSWORD;
    if (strcmp(text, "readonly") == 0) return MODE_READONLY;
    if (strcmp(text, "uppercase") == 0) return MODE_UPPERCASE;
    if (strcmp(text, "lowercase") == 0) return MODE_LOWERCASE;
    if (strcmp(text, "oemconvert") == 0) return MODE_OEMCONVERT;
    if (strcmp(text, "wrong-runtime-class") == 0) return MODE_WRONG_RUNTIME_CLASS;
    if (strcmp(text, "ambiguous-parent") == 0) return MODE_AMBIGUOUS_PARENT;
    if (strcmp(text, "ambiguous-control") == 0) return MODE_AMBIGUOUS_CONTROL;
    if (strcmp(text, "exit") == 0) return MODE_EXIT;
    return 0;
}

static int append_event(const char *event) {
    char line[384];
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

static int text_equals(HWND control, const wchar_t *expected) {
    wchar_t buffer[512];
    buffer[0] = L'\0';
    int length = GetWindowTextW(control, buffer, (int)(sizeof(buffer) / sizeof(buffer[0])));
    if (length < 0) {
        return 0;
    }
    return wcscmp(buffer, expected) == 0;
}

static const wchar_t *initial_text_for_mode(void) {
    switch (mode) {
        case MODE_IMPOSTOR: return L"impostor";
        case MODE_SCOPED: return L"30";
        case MODE_NOOP: return L"same-value";
        case MODE_UNICODE: return L"old";
        case MODE_CLEAR: return L"clear-me";
        case MODE_REDACTION: return L"before";
        case MODE_UPPERCASE: return L"INITIAL";
        default: return L"initial";
    }
}

static const wchar_t *expected_text_for_mode(void) {
    switch (mode) {
        case MODE_SCOPED: return L"60";
        case MODE_NOOP: return L"same-value";
        case MODE_UNICODE: return L"Caf\u00e9 \u03a9 \U0001F3AE";
        case MODE_CLEAR: return L"";
        case MODE_REDACTION: return L"runtime-secret-sentinel-4a913";
        default: return initial_text_for_mode();
    }
}

static int is_success_mutation_mode(void) {
    return mode == MODE_SCOPED || mode == MODE_UNICODE || mode == MODE_CLEAR || mode == MODE_REDACTION;
}

static int is_rejected_mode(void) {
    return mode == MODE_HIDDEN
        || mode == MODE_DISABLED
        || mode == MODE_MULTILINE
        || mode == MODE_PASSWORD
        || mode == MODE_READONLY
        || mode == MODE_UPPERCASE
        || mode == MODE_LOWERCASE
        || mode == MODE_OEMCONVERT
        || mode == MODE_WRONG_RUNTIME_CLASS
        || mode == MODE_AMBIGUOUS_PARENT
        || mode == MODE_AMBIGUOUS_CONTROL;
}

static void schedule_settle_close(HWND window) {
    KillTimer(window, TIMER_CLOSE);
    if (SetTimer(window, TIMER_CLOSE, 350, NULL) == 0) {
        event_failure = 1;
        DestroyWindow(window);
    }
}

static void monitor_state(void) {
    if (target_update_count > 1 || target_change_count > 1) {
        record_violation("VIOLATION-edit-target-notified-more-than-once");
    }
    if (wrong_id_update_count != 0 || wrong_id_change_count != 0) {
        record_violation("VIOLATION-edit-correct-class-wrong-id-notified");
    }
    if (other_window_update_count != 0 || other_window_change_count != 0) {
        record_violation("VIOLATION-edit-other-window-notified");
    }

    if (wrong_id_edit != NULL && IsWindow(wrong_id_edit) && !text_equals(wrong_id_edit, L"wrong-id")) {
        record_violation("VIOLATION-edit-correct-class-wrong-id-mutated");
    }
    if (other_window_edit != NULL && IsWindow(other_window_edit) && !text_equals(other_window_edit, L"other-window")) {
        record_violation("VIOLATION-edit-other-window-mutated");
    }

    if (target_edit == NULL || !IsWindow(target_edit)) {
        return;
    }

    if (mode == MODE_IMPOSTOR) {
        if (!text_equals(target_edit, L"impostor") || target_update_count != 0 || target_change_count != 0) {
            record_violation("VIOLATION-edit-other-process-mutated");
        }
        return;
    }
    if (mode == MODE_NOOP) {
        if (!text_equals(target_edit, expected_text_for_mode()) || target_update_count != 0 || target_change_count != 0) {
            record_violation("VIOLATION-edit-noop-mutated");
        }
        return;
    }
    if (is_rejected_mode()) {
        if (!text_equals(target_edit, initial_text_for_mode()) || target_update_count != 0 || target_change_count != 0) {
            record_violation("VIOLATION-edit-rejected-control-mutated");
        }
        if (duplicate_edit != NULL && IsWindow(duplicate_edit) && !text_equals(duplicate_edit, L"duplicate")) {
            record_violation("VIOLATION-edit-ambiguous-duplicate-mutated");
        }
        return;
    }
    if (is_success_mutation_mode() && target_change_count == 1 && !text_equals(target_edit, expected_text_for_mode())) {
        record_violation("VIOLATION-edit-target-change-result-mismatch");
    }
}

static void record_final_state(void) {
    monitor_state();
    if (mode == MODE_IMPOSTOR
        && target_edit != NULL
        && text_equals(target_edit, L"impostor")
        && target_update_count == 0
        && target_change_count == 0) {
        record_event("edit-other-process-final-unchanged");
    } else if (mode == MODE_SCOPED) {
        if (target_edit != NULL
            && text_equals(target_edit, L"60")
            && target_update_count == 1
            && target_change_count == 1) {
            record_event("edit-target-final-utf16-2-update-1-change-1");
        }
        if (wrong_id_edit != NULL && text_equals(wrong_id_edit, L"wrong-id")) {
            record_event("edit-correct-class-wrong-id-final-unchanged");
        }
        if (other_window_edit != NULL && text_equals(other_window_edit, L"other-window")) {
            record_event("edit-other-window-final-unchanged");
        }
    } else if (mode == MODE_NOOP
               && target_edit != NULL
               && text_equals(target_edit, L"same-value")
               && target_update_count == 0
               && target_change_count == 0) {
        record_event("edit-noop-final-utf16-10-update-0-change-0");
    } else if (mode == MODE_UNICODE
               && target_edit != NULL
               && text_equals(target_edit, expected_text_for_mode())
               && target_update_count == 1
               && target_change_count == 1) {
        record_event("edit-unicode-final-utf16-9-update-1-change-1");
    } else if (mode == MODE_CLEAR
               && target_edit != NULL
               && text_equals(target_edit, L"")
               && target_update_count == 1
               && target_change_count == 1) {
        record_event("edit-clear-final-utf16-0-update-1-change-1");
    } else if (mode == MODE_REDACTION
               && target_edit != NULL
               && text_equals(target_edit, expected_text_for_mode())
               && target_update_count == 1
               && target_change_count == 1) {
        record_event("edit-redaction-final-utf16-29-update-1-change-1");
    }
}

static LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
        case WM_COMMAND: {
            UINT code = HIWORD(wparam);
            HWND source = (HWND)lparam;
            int control_id = LOWORD(wparam);
            if (code != EN_UPDATE && code != EN_CHANGE) {
                break;
            }
            if (suppress_notifications) {
                return 0;
            }
            if (source == target_edit && control_id == target_control_id) {
                if (code == EN_UPDATE) {
                    target_update_count++;
                    if (target_update_count == 1) {
                        record_event("edit-target-en-update-1");
                    }
                } else {
                    target_change_count++;
                    if (target_change_count == 1) {
                        record_event("edit-target-en-change-1");
                    }
                    if (is_success_mutation_mode()
                        && target_update_count == 1
                        && target_change_count == 1
                        && text_equals(target_edit, expected_text_for_mode())) {
                        schedule_settle_close(target_window);
                    }
                }
                monitor_state();
                return 0;
            }
            if (source == wrong_id_edit && control_id == target_control_id + 1) {
                if (code == EN_UPDATE) wrong_id_update_count++;
                if (code == EN_CHANGE) wrong_id_change_count++;
                record_violation("VIOLATION-edit-correct-class-wrong-id-notification");
                return 0;
            }
            if (source == other_window_edit && control_id == target_control_id) {
                if (code == EN_UPDATE) other_window_update_count++;
                if (code == EN_CHANGE) other_window_change_count++;
                record_violation("VIOLATION-edit-other-window-notification");
                return 0;
            }
            if (source == duplicate_edit && control_id == target_control_id) {
                record_violation("VIOLATION-edit-ambiguous-duplicate-notification");
                return 0;
            }
            break;
        }
        case WM_TIMER:
            if (window != target_window) {
                return 0;
            }
            if (wparam == TIMER_SHOW_DISABLED && mode == MODE_SCOPED) {
                KillTimer(window, TIMER_SHOW_DISABLED);
                EnableWindow(target_edit, FALSE);
                ShowWindow(target_edit, SW_SHOW);
                UpdateWindow(target_edit);
                if (!text_equals(target_edit, L"30") || target_update_count != 0 || target_change_count != 0) {
                    record_violation("VIOLATION-edit-mutated-before-visible-disabled-stage");
                }
                record_event("edit-target-visible-disabled-utf16-2-update-0-change-0");
                if (SetTimer(window, TIMER_ENABLE_TARGET, enable_delay_ms, NULL) == 0) {
                    event_failure = 1;
                    DestroyWindow(window);
                }
                return 0;
            }
            if (wparam == TIMER_ENABLE_TARGET && mode == MODE_SCOPED) {
                KillTimer(window, TIMER_ENABLE_TARGET);
                if (!text_equals(target_edit, L"30") || target_update_count != 0 || target_change_count != 0) {
                    record_violation("VIOLATION-edit-mutated-before-enabled-stage");
                }
                EnableWindow(target_edit, TRUE);
                record_event("edit-target-visible-enabled-utf16-2-update-0-change-0");
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
        case WM_DESTROY:
            if (window == target_window) {
                if (other_window != NULL && IsWindow(other_window)) {
                    DestroyWindow(other_window);
                }
                PostQuitMessage(0);
            }
            return 0;
    }
    return DefWindowProcW(window, message, wparam, lparam);
}

static HWND create_top_level(HINSTANCE instance, const wchar_t *class_name, const wchar_t *title) {
    return CreateWindowExW(
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

static HWND create_child(
    HWND parent,
    const wchar_t *class_name,
    DWORD style,
    int control_id,
    int x,
    int y
) {
    return CreateWindowExW(
        WS_EX_CLIENTEDGE,
        class_name,
        L"",
        WS_CHILD | WS_TABSTOP | style,
        x,
        y,
        240,
        28,
        parent,
        (HMENU)(INT_PTR)control_id,
        GetModuleHandleW(NULL),
        NULL
    );
}

static int initialize_text(HWND control, const wchar_t *text) {
    if (control == NULL) {
        return 1;
    }
    return SetWindowTextW(control, text) ? 0 : 1;
}

static DWORD target_style_for_mode(void) {
    switch (mode) {
        case MODE_SCOPED: return ES_NUMBER;
        case MODE_MULTILINE: return ES_MULTILINE | ES_AUTOVSCROLL;
        case MODE_PASSWORD: return ES_PASSWORD;
        case MODE_READONLY: return ES_READONLY;
        case MODE_UPPERCASE: return ES_UPPERCASE;
        case MODE_LOWERCASE: return ES_LOWERCASE;
        case MODE_OEMCONVERT: return ES_OEMCONVERT;
        default: return ES_AUTOHSCROLL;
    }
}

static int setup_windows(HINSTANCE instance, const wchar_t *window_title) {
    static const wchar_t class_name[] = L"TandemEditSmokeWindow";
    WNDCLASSW window_class;
    ZeroMemory(&window_class, sizeof(window_class));
    window_class.lpfnWndProc = window_proc;
    window_class.hInstance = instance;
    window_class.lpszClassName = class_name;
    window_class.hCursor = LoadCursorW(NULL, MAKEINTRESOURCEW(32512));
    if (!RegisterClassW(&window_class) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
        return 1;
    }

    target_window = create_top_level(instance, class_name, window_title);
    if (target_window == NULL) {
        return 1;
    }

    suppress_notifications = 1;

    if (mode == MODE_WRONG_RUNTIME_CLASS) {
        target_edit = create_child(target_window, L"Static", SS_LEFT, target_control_id, 40, 50);
    } else {
        target_edit = create_child(target_window, L"Edit", target_style_for_mode(), target_control_id, 40, 50);
    }
    if (initialize_text(target_edit, initial_text_for_mode()) != 0) {
        return 1;
    }

    if (mode == MODE_SCOPED) {
        wrong_id_edit = create_child(target_window, L"Edit", ES_AUTOHSCROLL, target_control_id + 1, 40, 95);
        if (initialize_text(wrong_id_edit, L"wrong-id") != 0) return 1;
        HWND wrong_class = create_child(target_window, L"Static", SS_LEFT, target_control_id, 320, 50);
        if (initialize_text(wrong_class, L"wrong-class") != 0) return 1;
        other_window = create_top_level(instance, class_name, L"Other Edit Window");
        if (other_window == NULL) return 1;
        other_window_edit = create_child(other_window, L"Edit", ES_AUTOHSCROLL, target_control_id, 40, 50);
        if (initialize_text(other_window_edit, L"other-window") != 0) return 1;
        ShowWindow(wrong_id_edit, SW_SHOW);
        ShowWindow(wrong_class, SW_SHOW);
        ShowWindow(other_window_edit, SW_SHOW);
        ShowWindow(other_window, SW_SHOW);
        UpdateWindow(other_window);
    } else if (mode == MODE_AMBIGUOUS_PARENT) {
        other_window = create_top_level(instance, class_name, window_title);
        if (other_window == NULL) return 1;
        other_window_edit = create_child(other_window, L"Edit", ES_AUTOHSCROLL, target_control_id, 40, 50);
        if (initialize_text(other_window_edit, L"other-window") != 0) return 1;
        ShowWindow(other_window_edit, SW_SHOW);
        ShowWindow(other_window, SW_SHOW);
        UpdateWindow(other_window);
    } else if (mode == MODE_AMBIGUOUS_CONTROL) {
        duplicate_edit = create_child(target_window, L"Edit", ES_AUTOHSCROLL, target_control_id, 40, 95);
        if (initialize_text(duplicate_edit, L"duplicate") != 0) return 1;
        ShowWindow(duplicate_edit, SW_SHOW);
    }

    target_update_count = 0;
    target_change_count = 0;
    wrong_id_update_count = 0;
    wrong_id_change_count = 0;
    other_window_update_count = 0;
    other_window_change_count = 0;
    suppress_notifications = 0;

    ShowWindow(target_window, SW_SHOW);
    if (mode != MODE_HIDDEN && mode != MODE_SCOPED) {
        ShowWindow(target_edit, SW_SHOW);
    }
    if (mode == MODE_DISABLED) {
        EnableWindow(target_edit, FALSE);
    }
    UpdateWindow(target_window);

    if (mode == MODE_SCOPED) {
        record_event("edit-scoped-start");
        record_event("edit-selector-decoys-ready");
        record_event("edit-target-hidden-utf16-2-update-0-change-0");
        record_event("edit-other-window-ready-unchanged");
        if (SetTimer(target_window, TIMER_SHOW_DISABLED, show_delay_ms, NULL) == 0) return 1;
    } else if (mode == MODE_IMPOSTOR) {
        record_event("edit-impostor-start");
        record_event("edit-impostor-ready-unchanged");
    } else if (mode == MODE_NOOP) {
        record_event("edit-noop-ready-utf16-10-update-0-change-0");
    } else if (mode == MODE_UNICODE) {
        record_event("edit-unicode-ready-utf16-3-update-0-change-0");
    } else if (mode == MODE_CLEAR) {
        record_event("edit-clear-ready-utf16-8-update-0-change-0");
    } else if (mode == MODE_REDACTION) {
        record_event("edit-redaction-ready-utf16-6-update-0-change-0");
    } else {
        record_event("edit-rejection-target-ready");
    }

    if (SetTimer(target_window, TIMER_MONITOR, 25, NULL) == 0
        || SetTimer(target_window, TIMER_CLOSE, lifetime_ms, NULL) == 0) {
        return 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 8) {
        return 2;
    }
    event_path = argv[1];
    mode = parse_mode(argv[2]);
    target_control_id = (int)parse_unsigned(argv[4]);
    show_delay_ms = parse_unsigned(argv[5]);
    enable_delay_ms = parse_unsigned(argv[6]);
    lifetime_ms = parse_unsigned(argv[7]);
    if (mode == 0 || target_control_id <= 0 || lifetime_ms == 0) {
        return 2;
    }
    if (mode == MODE_EXIT) {
        record_event("edit-exit-tool-start");
        Sleep(150);
        record_event("edit-exit-tool-end");
        return event_failure ? 3 : 0;
    }

    wchar_t window_title[256];
    int converted = MultiByteToWideChar(CP_UTF8, 0, argv[3], -1, window_title, 256);
    if (converted <= 0) {
        return 2;
    }

    HINSTANCE instance = GetModuleHandleW(NULL);
    if (setup_windows(instance, window_title) != 0) {
        return 3;
    }

    MSG message;
    while (GetMessageW(&message, NULL, 0, 0) > 0) {
        TranslateMessage(&message);
        DispatchMessageW(&message);
    }
    return event_failure ? 3 : 0;
}
