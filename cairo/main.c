#include <cairo/cairo.h>
#include <cairo/cairo-ft.h>
#include <cairo/cairo-pdf.h>
#include <harfbuzz/hb.h>
#include <harfbuzz/hb-ft.h>
#include <harfbuzz/hb-icu.h>
#include <fontconfig/fontconfig.h>
#include <stdio.h>

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

  double pt_size = 64.0;

  // Locate the font file for the Cantarell font. The name is a Fontconfig
  // pattern; for example, Cantarell bold is "Cantarell:bold".
  FcPattern* pat = FcNameParse("Cantarell");
  FcDefaultSubstitute(pat);
  FcResult result;
  FcPattern* match = FcFontMatch(0, pat, &result);
  FcChar8* font_fname = 0;
  FcPatternGetString(match, FC_FILE, 0, &font_fname);
  printf("Font: %s\n", font_fname);

  // Note: FT assertions below are ignored.
  FT_Library ft_library;
  FT_Init_FreeType(&ft_library);

  FT_Face ft_face;
  FT_New_Face(ft_library, font_fname, 0, &ft_face);

  // This size does not affect anything except Harfbuzz. Apparently, we must set
  // the horizontal and vertical dpi to 72, otherwise offsets are wrong. We
  // could set the size to the actual point size, but by setting it to 1000.0 we
  // can reuse the FT font for different sizes. Why 1000 and not 1? For some
  // reason, with 1 the offsets are 0; I expect Harfbuzz rounds to integers
  // somewhere. Why 1k and not 10k? I found that with 10k -- for Cantarell at
  // least -- all offsets are multiples of 10 at that size. This might have
  // something to do with Truetype units.
  double hb_scale = 1000.0;
  FT_Set_Char_Size(ft_face, 0, hb_scale, 72, 72);

  // Note: we could not destroy these as long as we needed font_fname, as the
  // match or pattern apparently owns it.
  FcPatternDestroy(match);
  FcPatternDestroy(pat);

  //hb_face_t* hb_face = hb_ft_face_create(ft_face, 0);
  hb_font_t* hb_font = hb_ft_font_create(ft_face, 0);

  hb_buffer_t* hb_buf = hb_buffer_create();

  hb_buffer_set_direction(hb_buf, HB_DIRECTION_LTR);

  const char* str = "hi, world";
  hb_buffer_add_utf8(hb_buf, str, strlen(str), 0, strlen(str));
  hb_shape(hb_font, hb_buf, 0 /* features */, 0 /* num_features */);

  unsigned int glyph_count;
  hb_glyph_info_t* glyph_infos = hb_buffer_get_glyph_infos(hb_buf, &glyph_count);
  hb_glyph_position_t* glyph_poss = hb_buffer_get_glyph_positions(hb_buf, &glyph_count);

  cairo_font_face_t* font = cairo_ft_font_face_create_for_ft_face(ft_face, 0);
  cairo_glyph_t glyphs[9];

  if (glyph_count > sizeof(glyphs) / sizeof(cairo_glyph_t)) {
    printf("too many glyphs\n");
    return 1;
  }

  double x = 128.0;
  double y = 256.0;

  for (int i = 0; i < glyph_count; i++) {
    glyphs[i].index = glyph_infos[i].codepoint;
    x += glyph_poss[i].x_offset * pt_size / hb_scale;
    y += glyph_poss[i].y_offset * pt_size / hb_scale;
    glyphs[i].x = x;
    glyphs[i].y = y;
    x += glyph_poss[i].x_advance * pt_size / hb_scale;
    y += glyph_poss[i].y_advance * pt_size / hb_scale;
    printf("codepoint: %d\n", glyph_infos[i].codepoint);
    printf("x offset, advance: %d, %d\n", glyph_poss[i].x_offset, glyph_poss[i].x_advance);
    printf("y offset, advance: %d, %d\n", glyph_poss[i].y_offset, glyph_poss[i].y_advance);
  }

  cairo_set_font_face(cr, font);
  cairo_set_font_size(cr, pt_size);
  cairo_show_glyphs(cr, glyphs, glyph_count);

  cairo_font_face_destroy(font);

  cairo_show_page(cr);

  cairo_destroy(cr);
  cairo_surface_destroy(surf);

  FT_Done_FreeType(ft_library);

  return 0;
}
