// Copyright 2019 Alex Tennant
#include <QtWidgets/QApplication>

#include <chrono>

#include "main_window.hpp"

int main(int argc, char **argv) {
  QApplication app(argc, argv);

  MainWindow main_window;
  main_window.show();

  /* double current_time = hires_time_in_seconds();
  double accumulator = 0;

  while (main_window.isVisible()) {
    QApplication::processEvents();

    const double new_time = hires_time_in_seconds();
    const double frame_time = new_time - current_time;

    current_time = new_time;
    accumulator += frame_time;

    while (accumulator >= DELTA_TIME) {
      main_window.update();

      accumulator -= DELTA_TIME;
    }
  }*/

  return app.exec();
}
