#pragma once

#include <Windows.h>

struct Resource {
	LPVOID data;
	DWORD size;
};

LPVOID allocate_memory(const SIZE_T size);
void free_memory(LPVOID memory);

void display_error(const LPCWSTR msg, ...);
void display_warning(const LPCWSTR msg, ...);

bool load_data_resource(const int resource, Resource* result);

bool file_exists(const LPCWSTR filename);
bool dir_exists(const LPCWSTR dirname);
bool read_file(const LPCWSTR filename, DWORD bytes_to_read, LPVOID buffer);
bool write_file(const LPCWSTR filename, const DWORD bytes_to_write, const LPVOID buffer);