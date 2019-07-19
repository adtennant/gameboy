// Copyright 2019 Alex Tennant
#include "core.hpp"

extern "C" {
void *gb_create();
void gb_destroy(void *);
void gb_load_rom(void *, const char *, char *);
void gb_run_frame(void *);
void gb_get_frame_buffer(void *, core::Shade *);
// void gb_get_debug_info(void *, void *);
}

namespace core {

Emulator::Emulator() : emulator(gb_create(), gb_destroy) {}

std::string Emulator::load_rom(const std::string &filename) {
  char title[16];
  gb_load_rom(emulator.get(), filename.c_str(), title);
  return std::string(title);
}

void Emulator::run_frame() { gb_run_frame(emulator.get()); }

std::vector<Shade> Emulator::get_framebuffer() {
  std::vector<Shade> frame_buffer(160 * 144);
  gb_get_frame_buffer(emulator.get(), frame_buffer.data());
  return frame_buffer;
}

/* debug::Debug Emulator::get_debug_info() {
  debug::Debug debug_info;
  gb_get_debug_info(emulator.get(), &debug_info);
  return debug_info;
}*/

}  // namespace core
