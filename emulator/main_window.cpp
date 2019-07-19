// Copyright 2019 Alex Tennant
#include "main_window.hpp"

#include <QtCore/QTimer>
#include <QtGui/QPixmap>
#include <QtWidgets/QFileDialog>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QMenuBar>

#include <algorithm>
#include <chrono>
#include <memory>
#include <string>
#include <vector>

#include "core.hpp"

namespace {
const double CPU_CYCLES_PER_SECOND = 4194304;
const double CPU_CYCLES_PER_FRAME = 70224;
const double DELTA_TIME = 1.0 / (CPU_CYCLES_PER_SECOND / CPU_CYCLES_PER_FRAME);

double hires_time_in_seconds() {
  return std::chrono::duration_cast<std::chrono::duration<double>>(
             std::chrono::steady_clock::now().time_since_epoch())
      .count();
}
}  // namespace

struct Color {
  Color() : r(0), g(0), b(0) {}

  explicit Color(core::Shade shade) {
    switch (shade) {
      case core::Shade::White:
        r = 155;
        g = 188;
        b = 15;
        break;
      case core::Shade::LightGrey:
        r = 139;
        g = 172;
        b = 15;
        break;
      case core::Shade::DarkGrey:
        r = 48;
        g = 98;
        b = 48;
        break;
      case core::Shade::Black:
        r = 15;
        g = 56;
        b = 15;
        break;
    }
  }

  uint8_t r;
  uint8_t g;
  uint8_t b;
};

MainWindow::MainWindow()
    : emulator(), current_time(hires_time_in_seconds()), image_label(this) {
  image_label.setSizePolicy(QSizePolicy::Ignored, QSizePolicy::Ignored);

  setCentralWidget(&image_label);
  resize();

  QAction *open_action = new QAction("Open", this);
  open_action->setShortcut(QKeySequence::Open);
  connect(open_action, &QAction::triggered, this, &MainWindow::open_rom);

  QMenu *file_menu = menuBar()->addMenu("File");
  file_menu->addAction(open_action);

  QAction *zoom_in_action = new QAction("Zoom In", this);
  zoom_in_action->setShortcut(QKeySequence::ZoomIn);
  connect(zoom_in_action, &QAction::triggered, this, &MainWindow::zoom_in);

  QAction *zoom_out_action = new QAction("Zoom Out", this);
  zoom_out_action->setShortcut(QKeySequence::ZoomOut);
  connect(zoom_out_action, &QAction::triggered, this, &MainWindow::zoom_out);

  QAction *zoom_reset_action = new QAction("Reset Zoom", this);
  zoom_reset_action->setShortcut(QKeySequence(Qt::CTRL + Qt::Key_0));
  connect(zoom_reset_action, &QAction::triggered, this,
          &MainWindow::zoom_reset);

  QMenu *zoom_menu = menuBar()->addMenu("Zoom");
  zoom_menu->addAction(zoom_in_action);
  zoom_menu->addAction(zoom_out_action);
  zoom_menu->addSeparator();
  zoom_menu->addAction(zoom_reset_action);
}

void MainWindow::timerEvent(QTimerEvent *event) {
  const double new_time = hires_time_in_seconds();
  const double frame_time = new_time - current_time;

  current_time = new_time;
  accumulator += frame_time;

  while (accumulator >= DELTA_TIME) {
    emulator.run_frame();

    std::vector<core::Shade> frame_buffer = emulator.get_framebuffer();

    std::vector<Color> pixels(160 * 144);
    std::transform(frame_buffer.begin(), frame_buffer.end(), pixels.begin(),
                   [](core::Shade pixel) -> Color { return Color(pixel); });

    QImage image = QImage(reinterpret_cast<uint8_t *>(pixels.data()), 160, 144,
                          QImage::Format_RGB888);
    QPixmap pixmap = QPixmap::fromImage(image).scaled(size());
    image_label.setPixmap(pixmap);

    accumulator -= DELTA_TIME;
  }
}

void MainWindow::open_rom() {
  std::string filename =
      QFileDialog::getOpenFileName(this, "Open ROM", "", "Gameboy ROMs (*.gb)")
          .toStdString();

  std::string title = emulator.load_rom(filename);
  setWindowTitle(QString::fromStdString(title));

  startTimer(0, Qt::TimerType::PreciseTimer);
}

void MainWindow::resize() {
  static const int BASE_WIDTH = 160;
  static const int BASE_HEIGHT = 144;

  setFixedSize(BASE_WIDTH * zoom, BASE_HEIGHT * zoom);
}

void MainWindow::zoom_in() {
  zoom = std::min(zoom + 1, MAX_ZOOM);
  resize();
}

void MainWindow::zoom_out() {
  zoom = std::max(zoom - 1, MIN_ZOOM);
  resize();
}

void MainWindow::zoom_reset() {
  zoom = BASE_ZOOM;
  resize();
}
