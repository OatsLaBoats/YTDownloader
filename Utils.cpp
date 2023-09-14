#include "Utils.h"

#include <stdarg.h>
#include <stdio.h>

static WCHAR s_buffer[4096];

LPVOID allocate_memory(const SIZE_T size) {
	return HeapAlloc(GetProcessHeap(), 0, size);
}

void free_memory(LPVOID memory) {
	HeapFree(GetProcessHeap(), 0, memory);
}

void display_error(const LPCWSTR msg, ...) {
	va_list vl;
	va_start(vl, msg);
	vswprintf_s(s_buffer, 4096, msg, vl);
	va_end(vl);

	MessageBoxExW(nullptr, s_buffer, L"Error", MB_ICONERROR | MB_OK, 0);
}

void display_warning(const LPCWSTR msg, ...) {
	va_list vl;
	va_start(vl, msg);
	vswprintf_s(s_buffer, 4096, msg, vl);
	va_end(vl);

	MessageBoxExW(nullptr, s_buffer, L"Error", MB_ICONWARNING | MB_OK, 0);
}

bool file_exists(const LPCWSTR filename) {
	const DWORD att = GetFileAttributesW(filename);
	return (att != INVALID_FILE_ATTRIBUTES) && !(att & FILE_ATTRIBUTE_DIRECTORY);
}

bool dir_exists(const LPCWSTR dirname) {
	const DWORD att = GetFileAttributesW(dirname);
	return (att != INVALID_FILE_ATTRIBUTES) && (att & FILE_ATTRIBUTE_DIRECTORY);
}

bool load_data_resource(const int resource, Resource* result) {
	const HRSRC info_block = FindResourceW(nullptr, MAKEINTRESOURCEW(resource), MAKEINTRESOURCEW(RT_RCDATA));
	if (info_block == nullptr) {
		const DWORD e = GetLastError();
		display_error(L"Failed to get resource info block handle\nError Code: %lu", e);
		return false;
	}

	const HGLOBAL handle = LoadResource(nullptr, info_block);
	if (handle == nullptr) {
		const DWORD e = GetLastError();
		display_error(L"Failed to get resource handle\nError Code: %lu", e);
		return false;
	}

	const DWORD size = SizeofResource(nullptr, info_block);
	if (size == 0) {
		const DWORD e = GetLastError();
		display_error(L"Failed to get resource size\nError Code: %lu", e);
		return false;
	}

	const LPVOID data = LockResource(handle);
	if (data == nullptr) {
		const DWORD e = GetLastError();
		display_error(L"Failed to get resource data\nError Code: %lu", e);
		return false;
	}

	result->data = data;
	result->size = size;

	return true;
}

bool read_file(const LPCWSTR filename, const DWORD bytes_to_read, LPVOID buffer) {
	HANDLE file = CreateFileW(filename, GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
	if (file == INVALID_HANDLE_VALUE) {
		const DWORD e = GetLastError();
		display_error(L"Failed to get read handle for file %ls\nError Code: %lu", filename, e);
		return false;
	}

	DWORD read;
	if (!ReadFile(file, buffer, bytes_to_read, &read, nullptr)) {
		const DWORD e = GetLastError();
		display_error(L"Failed to read file %ls\nError Code: %lu", filename, e);
		CloseHandle(file);
		return false;
	}

	CloseHandle(file);

	return true;
}

bool write_file(const LPCWSTR filename, const DWORD bytes_to_write, const LPVOID buffer) {
	HANDLE file = CreateFileW(filename, GENERIC_WRITE, 0, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
	if (file == INVALID_HANDLE_VALUE) {
		const DWORD e = GetLastError();
		display_error(L"Failed to get write handle for file %ls\nError Code: %lu", filename, e);
		return false;
	}

	DWORD written;
	if (!WriteFile(file, buffer, bytes_to_write, &written, nullptr)) {
		const DWORD e = GetLastError();
		display_error(L"Failed to write to file %ls\nError Code: %lu", filename, e);
		CloseHandle(file);
		return false;
	}

	CloseHandle(file);

	return true;
}