#include <windows.h>
#include <stdio.h>

static DWORD parse_delay(const char *text) {
    DWORD value = 0;

    while (*text >= '0' && *text <= '9') {
        value = (value * 10) + (DWORD)(*text - '0');
        text++;
    }

    return value;
}

static int append_event(const char *path, const char *prefix, const char *suffix) {
    char line[256];
    int length = snprintf(line, sizeof(line), "%s-%s\r\n", prefix, suffix);
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

static LRESULT CALLBACK window_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    (void)wparam;
    (void)lparam;

    switch (message) {
        case WM_TIMER:
            DestroyWindow(window);
            return 0;
        case WM_CLOSE:
            DestroyWindow(window);
            return 0;
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
        default:
            return DefWindowProcA(window, message, wparam, lparam);
    }
}

int main(int argc, char **argv) {
    if (argc != 6) {
        return 2;
    }

    if (append_event(argv[1], argv[2], "start") != 0) {
        return 3;
    }

    Sleep(parse_delay(argv[4]));

    HINSTANCE instance = GetModuleHandleA(NULL);
    WNDCLASSA window_class = {0};
    window_class.lpfnWndProc = window_proc;
    window_class.hInstance = instance;
    window_class.lpszClassName = "TandemWindowSmokeHelper";

    if (!RegisterClassA(&window_class) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
        return 4;
    }

    HWND window = CreateWindowExA(
        0,
        window_class.lpszClassName,
        "Tandem Window Pending",
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        480,
        240,
        NULL,
        NULL,
        instance,
        NULL
    );
    if (window == NULL) {
        return 5;
    }

    ShowWindow(window, SW_SHOW);
    UpdateWindow(window);

    // Record readiness before assigning the matching title so event ordering proves that
    // Tandem waited for this process rather than the competing same-title process.
    if (append_event(argv[1], argv[2], "window") != 0) {
        DestroyWindow(window);
        return 6;
    }

    if (!SetWindowTextA(window, argv[3])) {
        DestroyWindow(window);
        return 7;
    }

    if (SetTimer(window, 1, parse_delay(argv[5]), NULL) == 0) {
        DestroyWindow(window);
        return 8;
    }

    MSG message;
    while (GetMessageA(&message, NULL, 0, 0) > 0) {
        TranslateMessage(&message);
        DispatchMessageA(&message);
    }

    return 0;
}
