#include <Windows.h>
#include <compressapi.h>
#include <Shlobj.h>
#include <stdio.h>
#include <string.h>

#include "resource1.h"
#include "Utils.h"

#define IDT_PROCESS_CHECK 1
#define IDT_ANIMATION 2

#define return_error { error_code = -1; goto done; }

static HWND window = nullptr;
static HWND edit_box = nullptr;
static HWND video_button = nullptr;
static HWND audio_button = nullptr;
static HWND status_label = nullptr;

static WCHAR app_path[MAX_PATH + 1] = {};
static WCHAR ytdlp_path[MAX_PATH + 1] = {};
static WCHAR ffmpeg_path[MAX_PATH + 1] = {};
static WCHAR version_path[MAX_PATH + 1] = {};

static WCHAR link[2048] = {};

static PWSTR download_path = nullptr;
static PWSTR appdata_path = nullptr;

static bool download_in_progress = false;
static PROCESS_INFORMATION process;

static int animation_frame = 0;

static bool extract_tool(const DECOMPRESSOR_HANDLE decompressor, const LPCWSTR path, const int resource_id) {
	if (!file_exists(path)) {
		Resource rs;
		if (!load_data_resource(resource_id, &rs)) return false;

		SIZE_T buf_size = 0;
		if (!Decompress(decompressor, rs.data, rs.size, NULL, 0, &buf_size)) {
			const DWORD e = GetLastError();
			if (e != ERROR_INSUFFICIENT_BUFFER) {
				display_error(L"Failed to get decompressed buffer size for %ls\nError Code: %lu", path, e);
				return false;
			}
		}

		PVOID buf = allocate_memory(buf_size);
		SIZE_T buf_data_size = 0;

		if (!Decompress(decompressor, rs.data, rs.size, buf, buf_size, &buf_data_size)) {
			const DWORD e = GetLastError();
			display_error(L"Can't decompress %ls\nError Code: %lu", path, e);
			return false;
		}

		if (!write_file(path, buf_data_size, buf)) {
			free_memory(buf);
			return false;
		}

		free_memory(buf);
	}

	return true;
}

static LRESULT CALLBACK window_proc(const HWND hwnd, const UINT msg, const WPARAM wparam, const LPARAM lparam) {
	switch (msg) {
	    case WM_DESTROY: {
            PostQuitMessage(0);
            return 0;
        } break;

		case WM_TIMER: {
			if (wparam == IDT_ANIMATION) {
				animation_frame += 1;

				if (animation_frame == 3) {
					animation_frame = 0;
				}
			}

			if (wparam == IDT_PROCESS_CHECK) {

			}

			return 0;
		} break;

		case WM_COMMAND: {
			// If the lparam is 0 then its not a message from a control.
			if (lparam != 0) {
				const WORD notification = HIWORD(wparam);
				const WORD id = LOWORD(wparam);
				const HWND handle = (HWND)lparam;

				switch (notification) {
					case BN_CLICKED: {
						if (download_in_progress) {
							display_warning(L"A download is already in progress,\nwait before staring another one.");
							return 0;
						}

						// Start timer for the main loop
						SetTimer(window, IDT_PROCESS_CHECK, 100, nullptr);

						static WCHAR command[4096];
						const int length = GetWindowTextW(edit_box, link, 2048);
						SetWindowTextW(edit_box, L"");

						if (length == 0) return 0;

						STARTUPINFOW si = {
							.cb = sizeof(si),
						};

						memset(&process, 0, sizeof(process));

						if (handle == video_button) {
							// For some reason it needs a space at the beginning otherwise it fails
							swprintf_s(command, 4096, L" --no-mtime --ffmpeg-location \"%ls\" --recode-video mp4 \"%ls\"", ffmpeg_path, link);

							BOOL success = CreateProcessW(
								ytdlp_path,
								command,
								nullptr,
								nullptr,
								FALSE,
								NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW,
								nullptr,
								download_path,
								&si,
								&process
							);

							if (!success) {
								const DWORD e = GetLastError();
								display_error(L"Failed to launch process\nError Code: %lu", e);
								return 0;
							}

							download_in_progress = true;
						}
						else if (handle == audio_button) {
							// For some reason it needs a space at the beginning otherwise it fails
							swprintf_s(command, 4096, L" --no-mtime --ffmpeg-location \"%ls\" -x --audio-format mp3  --audio-quality 0 \"%ls\"", ffmpeg_path, link);

							BOOL success = CreateProcessW(
								ytdlp_path,
								command,
								nullptr,
								nullptr,
								FALSE,
								NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW,
								nullptr,
								download_path,
								&si,
								&process
							);

							if (!success) {
								const DWORD e = GetLastError();
								display_error(L"Failed to launch process\nError Code: %lu", e);
								return 0;
							}

							download_in_progress = true;
						}

						return 0;
					} break;
				}
			}
		} break;
	}

	return DefWindowProcW(hwnd, msg, wparam, lparam);
}

int WINAPI wWinMain(const HINSTANCE instance, const HINSTANCE prev_instance, const PWSTR cmd_line, const int cmd_show) {
	const char* version = "00001";
	const int version_length = 5;

	const LPCWSTR window_class_name = L"YTDownloaderWindowClass";
	const int window_width = 400;
	const int window_height = 254;
	const int button_width = (window_width - 40) / 2 - 5;

	WNDCLASSEXW window_class;
	ATOM window_class_id;

	HWND edit_box_label;
	HWND buttons_label;

	MSG msg = {};

	bool should_update = false;
	bool should_update_status = true;
	int last_frame_i = -1;
	int error_code = 0;

	// Variables that need to be cleaned up
	DECOMPRESSOR_HANDLE decompressor = nullptr;

	if (SHGetKnownFolderPath(FOLDERID_Downloads, KF_FLAG_DEFAULT, nullptr, &download_path) != S_OK) {
		display_error(L"Failed to get dowloads folder path");
		return_error;
	}

	if (SHGetKnownFolderPath(FOLDERID_LocalAppData, KF_FLAG_DEFAULT, nullptr, &appdata_path) != S_OK) {
		display_error(L"Failed to get AppData\\Local folder path");
		return_error;
	}

	// Create all the necessary paths
	swprintf_s(app_path, MAX_PATH + 1, L"%ls\\YT Downloader", appdata_path);
	swprintf_s(ytdlp_path, MAX_PATH + 1, L"%ls\\YT Downloader\\yt-dlp.exe", appdata_path);
	swprintf_s(ffmpeg_path, MAX_PATH + 1, L"%ls\\YT Downloader\\ffmpeg.exe", appdata_path);
	swprintf_s(version_path, MAX_PATH + 1, L"%ls\\YT Downloader\\version", appdata_path);

	// Check if appdata directory exists
	if (!dir_exists(app_path)) {
		if (!CreateDirectoryW(app_path, nullptr)) {
			display_error(L"Failed to create app directory");
			return_error;
		}
	}

	// Check the version file incase there is a need to update the cached ffmpeg and yt-dlp
	if (!file_exists(version_path)) {
		should_update = true;
		if (!write_file(version_path, version_length, (LPVOID)version)) return_error;
	}
	else {
		char buf[version_length];
		if (!read_file(version_path, version_length, buf)) return_error;

		if (memcmp(buf, version, version_length) != 0) {
			should_update = true;
			if (!write_file(version_path, version_length, (LPVOID)version)) return_error;
		}
	}

	if (should_update) {
		DeleteFileW(ytdlp_path);
		DeleteFileW(ffmpeg_path);
	}

	if (!CreateDecompressor(COMPRESS_ALGORITHM_LZMS, nullptr, (PDECOMPRESSOR_HANDLE)&decompressor)) {
		const DWORD e = GetLastError();
		display_error(L"Failed to initialize decompressor\nError Code: %lu", e);
		return_error;
	}

	if (!extract_tool(decompressor, ytdlp_path, IDR_RCDATA1)) return_error;
	if (!extract_tool(decompressor, ffmpeg_path, IDR_RCDATA2)) return_error;

	CloseDecompressor(decompressor);
	decompressor = nullptr;

	window_class.cbSize = sizeof(window_class),
	window_class.style = CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
	window_class.lpfnWndProc = window_proc,
	window_class.cbClsExtra = 0,
	window_class.cbWndExtra = 0,
	window_class.hInstance = instance,
	window_class.hIcon = LoadIconW(instance, MAKEINTRESOURCEW(IDI_ICON1)),
	window_class.hCursor = nullptr,
	window_class.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1),
	window_class.lpszMenuName = nullptr,
	window_class.lpszClassName = window_class_name,
	window_class.hIconSm = nullptr,

	window_class_id = RegisterClassExW(&window_class);

	if (window_class_id == 0) {
		const DWORD e = GetLastError();
		display_error(L"Failed to initialize window class\nError Code: %lu", e);
		return_error;
	}

	window = CreateWindowExW(
		WS_EX_OVERLAPPEDWINDOW,
		window_class_name,
		L"YT Downloader",
		WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE,
		CW_USEDEFAULT, CW_USEDEFAULT,
		window_width, window_height,
		nullptr,
		nullptr,
		instance,
		nullptr
	);

	if (window == nullptr) {
		const DWORD e = GetLastError();
		display_error(L"Failed to initialize window\nError Code: %lu", e);
		return_error;
	}

	edit_box_label = CreateWindowExW(
		0,
		L"STATIC",
		L"Enter link:",
		WS_CHILD | SS_CENTER | WS_VISIBLE,
		10, 10,
		window_width - 40, 20,
		window,
		nullptr,
		instance,
		nullptr
	);

	edit_box = CreateWindowExW(
		0,
		L"EDIT",
		L"",
		WS_TABSTOP | WS_CHILD | WS_BORDER | ES_AUTOHSCROLL | WS_VISIBLE,
		10, 30,
		window_width - 40, 20,
		window,
		nullptr,
		instance,
		nullptr
	);

	buttons_label = CreateWindowExW(
		0,
		L"STATIC",
		L"Select download format:",
		WS_CHILD | SS_CENTER | WS_VISIBLE,
		10, 80,
		window_width - 40, 20,
		window,
		nullptr,
		instance,
		nullptr
	);

	video_button = CreateWindowExW(
		0,
		L"BUTTON",
		L"Video",
		WS_TABSTOP | WS_CHILD | BS_DEFPUSHBUTTON | WS_VISIBLE,
		10, 108,
		button_width, 50,
		window,
		nullptr,
		instance,
		nullptr
	);

	audio_button = CreateWindowExW(
		0,
		L"BUTTON",
		L"Audio",
		WS_TABSTOP | WS_CHILD | BS_DEFPUSHBUTTON | WS_VISIBLE,
		10 + button_width + 10, 108,
		button_width, 50,
		window,
		nullptr,
		instance,
		nullptr
	);

	status_label = CreateWindowExW(
		0,
		L"STATIC",
		L"",
		WS_CHILD | SS_CENTER,
		10, 166,
		window_width - 40, 35,
		window,
		nullptr,
		instance,
		nullptr
	);

	while (true) {
		BOOL ret = GetMessageW(&msg, nullptr, 0, 0);
		if (ret == 0) break;

		if (ret == -1) {
			const DWORD e = GetLastError();
			display_error(L"GetMessageW error\nError Code %lu", e);
			return_error;
		}

		// Checks if the download is finished
		if (download_in_progress) {
			DWORD code;
			if (!GetExitCodeProcess(process.hProcess, &code)) {
				const DWORD e = GetLastError();
				display_error(L"Failed to get process exit code\nError Code: %lu", e);

				TerminateProcess(process.hProcess, -1);
				CloseHandle(process.hProcess);
				KillTimer(window, IDT_PROCESS_CHECK);
				KillTimer(window, IDT_ANIMATION);

				ShowWindow(status_label, SW_HIDE);
				SetWindowTextW(status_label, L"");

				download_in_progress = false;
				should_update_status = true;
				animation_frame = 0;
				last_frame_i = -1;
			}
			else if (code != STILL_ACTIVE) {
				download_in_progress = false;
				should_update_status = true;
				animation_frame = 0;
				last_frame_i = -1;

				CloseHandle(process.hProcess);
				KillTimer(window, IDT_PROCESS_CHECK);
				KillTimer(window, IDT_ANIMATION);

				if (code == 0) {
					static WCHAR buf[MAX_PATH];
					swprintf_s(buf, MAX_PATH, L"Download finished\nFile location: \"%ls\"", download_path);
					SetWindowTextW(status_label, buf);
				}
				else {
					display_error(L"Failed to download \"%ls\"", link);
					ShowWindow(status_label, SW_HIDE);
					SetWindowTextW(status_label, L"");
				}
			}
			else {
				if (should_update_status) {
					SetTimer(window, IDT_ANIMATION, 500, nullptr);
					ShowWindow(status_label, SW_SHOW);
					should_update_status = false;
				}

				if (animation_frame != last_frame_i) {
					last_frame_i = animation_frame;

					const LPCWSTR frames[3] = {
						L"Downloading.",
						L"Downloading..",
						L"Downloading...",
					};

					const LPCWSTR frame = frames[animation_frame];

					SetWindowTextW(status_label, frame);
				}
			}
		}

		TranslateMessage(&msg);
		DispatchMessageW(&msg);
	}

	done:

	if (decompressor != nullptr) { 
		CloseDecompressor(decompressor); 
	}

	if (download_path != nullptr) {
		CoTaskMemFree(download_path);
	}

	if (appdata_path != nullptr) {
		CoTaskMemFree(appdata_path);
	}

	return error_code;
}