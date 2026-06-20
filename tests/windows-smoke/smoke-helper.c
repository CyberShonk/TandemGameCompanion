#include <windows.h>

static DWORD parse_delay(const char *text) {
    DWORD value = 0;

    while (*text >= '0' && *text <= '9') {
        value = (value * 10) + (DWORD)(*text - '0');
        text++;
    }

    return value;
}

static int append_line(const char *path, const char *text) {
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

    if (!WriteFile(file, text, (DWORD)lstrlenA(text), &written, NULL)) {
        CloseHandle(file);
        return 1;
    }

    if (!WriteFile(file, "\r\n", 2, &written, NULL)) {
        CloseHandle(file);
        return 1;
    }

    CloseHandle(file);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 4) {
        return 2;
    }

    if (append_line(argv[1], argv[2]) != 0) {
        return 3;
    }

    Sleep(parse_delay(argv[3]));

    if (argc >= 5 && append_line(argv[1], argv[4]) != 0) {
        return 4;
    }

    return 0;
}
