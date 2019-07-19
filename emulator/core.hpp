// Copyright 2019 Alex Tennant
#ifndef EMULATOR_CORE_HPP_
#define EMULATOR_CORE_HPP_

#include <memory>
#include <string>
#include <vector>

namespace core {
enum struct Shade : uint8_t {
  White = 0,
  LightGrey = 1,
  DarkGrey = 2,
  Black = 3
};

class Emulator {
 public:
  Emulator();
  std::string load_rom(const std::string &filename);
  void run_frame();

  std::vector<Shade> get_framebuffer();

 private:
  std::unique_ptr<void, void (*)(void *)> emulator;
};
}  // namespace core

#endif  // EMULATOR_CORE_HPP_
