#include <cairo/cairo.h>
#include <cairo/cairo-pdf.h>

int main(int argc, char** argv)
{
  double width = 1920.0;
  double height = 1080.0;
  cairo_surface_t* surf = cairo_pdf_surface_create("test.pdf", width, height);
  cairo_t* cr = cairo_create(surf);

  cairo_set_source_rgb(cr, 0.0, 0.0, 0.0);
  cairo_set_line_width(cr, 6.0);

  cairo_move_to(cr, 32.0, 32.0);
  cairo_line_to(cr, 960.0, 520.0);

  cairo_stroke(cr);

  cairo_show_page(cr);

  cairo_destroy(cr);
  cairo_surface_destroy(surf);

  return 0;
}
