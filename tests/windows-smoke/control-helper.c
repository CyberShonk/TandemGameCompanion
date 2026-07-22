#include <windows.h>
#include <stdio.h>
#include <string.h>

#define TIMER_SHOW_DISABLED 1
#define TIMER_ENABLE_CONTROL 2
#define TIMER_CLOSE_WINDOWS 3

static const char *event_path = NULL;
static HWND target_window = NULL;
static HWND other_window = NULL;
static HWND target_control = NULL;
static DWORD disabled_delay_ms = 0;
static int event_failure = 0;

static DWORD parse_delay(const char *text) {
    DWORD value = 0;

    while (*text >= '0' && *text <= '9') {
        value = (value * 10) + (DWORD)(*text - '0');
        text++;
    }

    return value;
}

static int parse_control_id(const char *text) {
    int value = 0;

    while (*text >= '0' && *text <= '9') {
        value = (value * 10) + (*text - '0');
        text++;
    }

    return value;
}

static int append_event(const char *path, const char *event) {
    char line[256];
    int length = snprintf(line, sizeof(line), "%s\r\n", event);
    if (length < 0 || length >= (int)sizeof(line)) {
        return 1;
    }

    HANDLE file = CreateFileA(
        path,
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

static void fail_event_write(void) {
    event_failure = 1;
    if (target_window != NULL) {
        DestroyWindow(target_window);
    }
}

static LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    (void)wparam;
    (void)lparam;

    switch (message) {
        case WM_TIMER:
            if (window != target_window) {
                return 0;
            }

            if (wparam == TIMER_SHOW_DISABLED) {
                KillTimer(window, TIMER_SHOW_DISABLED);
                EnableWindow(target_control, FALSE);
                ShowWindow(target_control, SW_SHOW);
                UpdateWindow(target_control);
                if (append_event(event_path, "scoped-control-visible-disabled") != 0) {
                    fail_event_write();
                    return 0;
                }
                if (SetTimer(
                        window,
                        TIMER_ENABLE_CONTROL,
                        disabled_delay_ms,
                        NULL
                    ) == 0) {
                    event_failure = 1;
                    DestroyWindow(window);
                }
                return 0;
            }

            if (wparam == TIMER_ENABLE_CONTROL) {
                KillTimer(window, TIMER_ENABLE_CONTROL);
                EnableWindow(target_control, TRUE);
                if (append_event(event_path, "scoped-control-visible-enabled") != 0) {
                    fail_event_write();
                }
                return 0;
            }

            if (wparam == TIMER_CLOSE_WINDOWS) {
                DestroyWindow(window);
                return 0;
            }
            return 0;

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
            return DefWindowProcA(window, message, wparam, lparam);
    }
}

static HWND create_top_level_window(
    HINSTANCE instance,
    const char *class_name,
    const char *title
) {
    return CreateWindowExA(
        0,
        class_name,
        title,
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        520,
        280,
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
        220,
        32,
        parent,
        (HMENU)(INT_PTR)control_id,
        GetModuleHandleA(NULL),
        NULL
    );
}

static int run_impostor(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    int control_id,
    DWORD lifetime_ms
) {
    if (append_event(event_path, "control-impostor-start") != 0) {
        return 3;
    }

    target_window = create_top_level_window(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }

    target_control = create_control(
        target_window,
        "ComboBox",
        WS_VISIBLE | CBS_DROPDOWNLIST,
        control_id,
        24,
        24
    );
    if (target_control == NULL) {
        DestroyWindow(target_window);
        return 5;
    }

    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);

    if (append_event(event_path, "control-impostor-control-ready") != 0) {
        DestroyWindow(target_window);
        return 6;
    }

    if (SetTimer(target_window, TIMER_CLOSE_WINDOWS, lifetime_ms, NULL) == 0) {
        DestroyWindow(target_window);
        return 7;
    }

    return 0;
}

static int run_scoped(
    HINSTANCE instance,
    const char *class_name,
    const char *title,
    int control_id,
    DWORD hidden_delay_ms,
    DWORD disabled_ms,
    DWORD lifetime_ms
) {
    if (append_event(event_path, "scoped-control-start") != 0) {
        return 3;
    }

    target_window = create_top_level_window(instance, class_name, title);
    if (target_window == NULL) {
        return 4;
    }

    HWND correct_id_wrong_class = create_control(
        target_window,
        "Button",
        WS_VISIBLE | BS_PUSHBUTTON,
        control_id,
        24,
        24
    );
    HWND correct_class_wrong_id = create_control(
        target_window,
        "ComboBox",
        WS_VISIBLE | CBS_DROPDOWNLIST,
        control_id + 1,
        24,
        72
    );
    target_control = create_control(
        target_window,
        "ComboBox",
        CBS_DROPDOWNLIST,
        control_id,
        24,
        120
    );
    if (correct_id_wrong_class == NULL
        || correct_class_wrong_id == NULL
        || target_control == NULL) {
        DestroyWindow(target_window);
        return 5;
    }

    ShowWindow(target_window, SW_SHOW);
    UpdateWindow(target_window);

    if (append_event(event_path, "scoped-control-selector-decoys-ready") != 0
        || append_event(event_path, "scoped-control-hidden") != 0) {
        DestroyWindow(target_window);
        return 6;
    }

    other_window = create_top_level_window(
        instance,
        class_name,
        "Different Trainer Top-Level Window"
    );
    if (other_window == NULL) {
        DestroyWindow(target_window);
        return 7;
    }

    HWND other_control = create_control(
        other_window,
        "ComboBox",
        WS_VISIBLE | CBS_DROPDOWNLIST,
        control_id,
        24,
        24
    );
    if (other_control == NULL) {
        DestroyWindow(target_window);
        return 8;
    }

    ShowWindow(other_window, SW_SHOW);
    UpdateWindow(other_window);

    if (append_event(event_path, "scoped-control-other-window-ready") != 0) {
        DestroyWindow(target_window);
        return 9;
    }

    disabled_delay_ms = disabled_ms;
    if (SetTimer(
            target_window,
            TIMER_SHOW_DISABLED,
            hidden_delay_ms,
            NULL
        ) == 0
        || SetTimer(
            target_window,
            TIMER_CLOSE_WINDOWS,
            lifetime_ms,
            NULL
        ) == 0) {
        DestroyWindow(target_window);
        return 10;
    }

    return 0;
}

int main(int argc, char **argv) {
    if (argc != 6 && argc != 8) {
        return 2;
    }

    event_path = argv[1];
    const char *mode = argv[2];
    const char *title = argv[3];
    int control_id = parse_control_id(argv[4]);

    HINSTANCE instance = GetModuleHandleA(NULL);
    WNDCLASSA window_class = {0};
    window_class.lpfnWndProc = window_proc;
    window_class.hInstance = instance;
    window_class.lpszClassName = "TandemControlSmokeHelper";

    if (!RegisterClassA(&window_class) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
        return 11;
    }

    int result;
    if (strcmp(mode, "impostor") == 0 && argc == 6) {
        result = run_impostor(
            instance,
            window_class.lpszClassName,
            title,
            control_id,
            parse_delay(argv[5])
        );
    } else if (strcmp(mode, "scoped") == 0 && argc == 8) {
        result = run_scoped(
            instance,
            window_class.lpszClassName,
            title,
            control_id,
            parse_delay(argv[5]),
            parse_delay(argv[6]),
            parse_delay(argv[7])
        );
    } else {
        return 2;
    }

    if (result != 0) {
        return result;
    }

    MSG message;
    while (GetMessageA(&message, NULL, 0, 0) > 0) {
        TranslateMessage(&message);
        DispatchMessageA(&message);
    }

    return event_failure ? 12 : 0;
}
