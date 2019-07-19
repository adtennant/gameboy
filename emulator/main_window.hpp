// Copyright 2019 Alex Tennant
#ifndef EMULATOR_MAIN_WINDOW_HPP_
#define EMULATOR_MAIN_WINDOW_HPP_

#include <QtWidgets/QLabel>
#include <QtWidgets/QMainWindow>

#include "core.hpp"

class MainWindow : public QMainWindow {
  Q_OBJECT

 public:
  MainWindow();

 protected:
  virtual void timerEvent(QTimerEvent *event);

 private:
  void open_rom();

  void resize();

  void show_vram();

  void zoom_in();
  void zoom_out();
  void zoom_reset();

  const int BASE_ZOOM = 1;
  const int MIN_ZOOM = 1;
  const int MAX_ZOOM = 8;

  core::Emulator emulator;
  double current_time;
  double accumulator = 0;

  int zoom = BASE_ZOOM;
  QLabel image_label;
};

#endif  // EMULATOR_MAIN_WINDOW_HPP_
