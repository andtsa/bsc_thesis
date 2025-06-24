library(ggplot2)
library(readr)
library(scales)
library(tidyverse)
library(sysfonts)
library(showtext)
library(dplyr)
library(viridis)

font_add(
    family = "LinLibertine",
    regular = "/Users/andtsa/Library/Fonts/LinLibertine_RB.otf",
    bold = "/Users/andtsa/Library/Fonts/LinLibertine_RB.otf"
)

showtext_auto()

plot_tau <- function(df, x, y, x_label, y_label, plot_title, out_file, diag_alpha, bins = 100) {
    if ("longest_tie" %in% colnames(df)) {
        df <- df %>% mutate(
            w = t_max - t_min,
            scaled_longest_tie = longest_tie / length
        )
    } else {
        df <- df %>%
            filter(!str_detect(err_type, "missing"))
    }
    p <- ggplot(df, aes(x = .data[[x]], y = .data[[y]])) +
        stat_bin2d(
            bins = bins,
            aes(fill = after_stat(count)),
            colour = NA
        ) +
        scale_fill_viridis_c(
            option = "magma",
            trans  = "sqrt", # for perceptual uniformity
            # guide  = "none" # guide_colorbar(barwidth = 0.5, barheight = 10)
        ) +
        scale_x_continuous(
            limits       = c(-1, 1),
            breaks       = seq(-1, 1, .5),
            minor_breaks = seq(-1, 1, .1),
            # expand       = expansion(add = .02)
        ) +
        scale_y_continuous(
            limits       = c(-1, 1),
            breaks       = seq(-1, 1, .5),
            minor_breaks = seq(-1, 1, .125),
            # expand       = expansion(add = .02)
        ) +
        geom_abline(intercept = 0, slope = 1, colour = "#ff2050", size = 0.8, alpha = diag_alpha) +
        labs(x = x_label, y = y_label, title = plot_title, fill = "Count") +
        theme_minimal(base_size = 24, base_family = "LinLibertine") +
        theme(
            panel.grid.major = element_line(colour = "grey80", linewidth = 0.6),
            panel.grid.minor = element_line(colour = "grey90", linewidth = 0.45),
            axis.title = element_text(face = "bold", size = 32),
            axis.text = element_text(colour = "grey20"),
            plot.title = element_text(face = "bold", hjust = 0.5, size = 32),
            legend.position = "none"
        )


    ggsave(out_file, p, width = 7, height = 7)
}

# do the stuff
plots <- tribble(
    ~data_file, ~dataset, ~x, ~y, ~x_label, ~y_label, ~plot_title, ~suffix, ~diag_alpha,
    "~/mytmp/data/synth.csv", "synthheat", "t_a", "t_max", "τ-a", "τ-max", "synthetic data: τ-max vs τ-a", "tmax_a", 0,
    "~/mytmp/data/synth.csv", "synthheat", "t_b", "t_max", "τ-b", "τ-max", "synthetic data: τ-max vs τ-b", "tmax_b", 0,
    "~/mytmp/data/synth.csv", "synthheat", "t_a", "t_min", "τ-a", "τ-min", "synthetic data: τ-min vs τ-a", "tmin_a", 0,
    "~/mytmp/data/synth.csv", "synthheat", "t_b", "t_min", "τ-b", "τ-min", "synthetic data: τ-min vs τ-b", "tmin_b", 0,
    # AP
    "~/mytmp/data/synth-ap.csv", "synthap", "t_a", "t_max", "τAP-a", "τ-max", "synthetic data: τ-max vs τAP-a", "tmax_a", 0.8,
    "~/mytmp/data/synth-ap.csv", "synthap", "t_b", "t_max", "τAP", "τ-max", "synthetic data: τ-max vs τAP-b", "tmax_b", 0.8,
    "~/mytmp/data/synth-ap.csv", "synthap", "t_a", "t_min", "τAP-a", "τ-min", "synthetic data: τ-min vs τAP-a", "tmin_a", 0.8,
    "~/mytmp/data/synth-ap.csv", "synthap", "t_b", "t_min", "τAP", "τ-min", "synthetic data: τ-min vs τAP-b", "tmin_b", 0.8,
    # trec
    "~/mytmp/data/trec-fixed.csv", "trec", "t_a", "t_max", "τ-a", "τ-max", "TREC data: τ-max vs τ-a", "tmax_a", 0.9,
    "~/mytmp/data/trec-fixed.csv", "trec", "t_b", "t_max", "τ-b", "τ-max", "TREC data: τ-max vs τ-b", "tmax_b", 0.9,
    "~/mytmp/data/trec-fixed.csv", "trec", "t_a", "t_min", "τ-a", "τ-min", "TREC data: τ-min vs τ-a", "tmin_a", 0.9,
    "~/mytmp/data/trec-fixed.csv", "trec", "t_b", "t_min", "τ-b", "τ-min", "TREC data: τ-min vs τ-b", "tmin_b", 0.9,
    # w/lt
    # "~/mytmp/data/synth.csv", "uncertainty", "scaled_longest_tie", "w", "longest tie / n", "W", "synthetic data: uncertainty w vs longest tie", "wlt",

    # approx
    "~/mytmp/data/th-comp.csv", "th", "ltmin", "rtmin", "τ-h-min found\nby Algorithm 2", "true τ-min", "", "approx_min", 0,
    "~/mytmp/data/th-comp.csv", "th", "ltmax", "rtmax", "τ-h-max found\nby Algorithm 2", "true τ-max", "", "approx_max", 0,
)


# add output path
plots <- plots %>%
    mutate(
        out_file = file.path(
            "~/cse3000/andreas/final/plots",
            paste0(dataset, "_", suffix, ".pdf")
        )
    )

# loop and plot
plots %>%
    pwalk(function(data_file, dataset, x, y, x_label, y_label, plot_title, suffix, out_file, diag_alpha) {
        df <- read_csv(data_file)
        # df <- df %>% mutate(scaled_longest_tie = longest_tie / length)
        plot_tau(df, x, y, x_label, y_label, plot_title, out_file, diag_alpha)
    })
