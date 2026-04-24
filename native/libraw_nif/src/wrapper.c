#include <libraw/libraw.h>

/* ---------- parameter setters ---------- */

void lrnif_set_output_bps(libraw_data_t *lr, int bps) {
    lr->params.output_bps = bps;
}

void lrnif_set_use_camera_wb(libraw_data_t *lr, int v) {
    lr->params.use_camera_wb = v;
}

void lrnif_set_no_auto_bright(libraw_data_t *lr, int v) {
    lr->params.no_auto_bright = v;
}

void lrnif_set_gamm(libraw_data_t *lr, double g0, double g1) {
    lr->params.gamm[0] = g0;
    lr->params.gamm[1] = g1;
}

/* ---------- metadata getters ---------- */

const char *lrnif_get_make(libraw_data_t *lr) {
    return lr->idata.make;
}

const char *lrnif_get_model(libraw_data_t *lr) {
    return lr->idata.model;
}

time_t lrnif_get_timestamp(libraw_data_t *lr) {
    return lr->other.timestamp;
}

float lrnif_get_iso(libraw_data_t *lr) {
    return lr->other.iso_speed;
}

float lrnif_get_shutter(libraw_data_t *lr) {
    return lr->other.shutter;
}

float lrnif_get_aperture(libraw_data_t *lr) {
    return lr->other.aperture;
}

int lrnif_get_flip(libraw_data_t *lr) {
    return lr->sizes.flip;
}

/* ---------- processed image accessors ---------- */

unsigned short lrnif_image_height(libraw_processed_image_t *img) {
    return img->height;
}

unsigned short lrnif_image_width(libraw_processed_image_t *img) {
    return img->width;
}

unsigned short lrnif_image_colors(libraw_processed_image_t *img) {
    return img->colors;
}

unsigned short lrnif_image_bits(libraw_processed_image_t *img) {
    return img->bits;
}

unsigned int lrnif_image_data_size(libraw_processed_image_t *img) {
    return img->data_size;
}

unsigned char *lrnif_image_data(libraw_processed_image_t *img) {
    return img->data;
}
