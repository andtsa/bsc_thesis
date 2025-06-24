library(ggplot2)
library(dplyr)
library(patchwork)
library(readr)
library(sysfonts)
library(showtext)
library(geomtextpath)
library(viridis)

font_add(
    family = "LinLibertine",
    regular = "/Users/andtsa/Library/Fonts/LinLibertine_RB.otf",
    bold = "/Users/andtsa/Library/Fonts/LinLibertine_RB.otf"
)
showtext_auto()


df <- read_csv("~/mytmp/data/synth.csv")

df_a <- df %>% filter(t_a >= 0.9)
df_b <- df %>% filter(t_b >= 0.9)

df2 <- df %>%
    filter(t_a >= 0.9 | t_b >= 0.9) %>%
    mutate(
        group = case_when(
            t_a >= 0.9 ~ "τ-a ≥ 0.9",
            t_b >= 0.9 ~ "τ-b ≥ 0.9"
        )
    )

cols <- c("#d02f42", "#4810a0")

p <- ggplot(df2, aes(x = t_min, fill = group)) +
    geom_histogram(
        bins     = 110,
        position = "identity",
        alpha    = 0.85,
        colour   = NA
    ) +
    scale_fill_manual(
        values = cols,
        name   = NULL
    ) +
    geom_vline(xintercept = 0.9, colour = "#201037", linetype = "longdash", linewidth = 0.9, show.legend = TRUE) +
    # geom_textvline(label = "aaaaaaaaa", xintercept = 0.9, vjust = 1.3) +
    geom_vline(xintercept = 0.8, colour = "#48194b", linetype = "longdash", linewidth = 0.9, show.legend = TRUE) +
    scale_x_continuous(
        # limits       = c(-1, 1),
        breaks       = seq(-1, 1, 0.1),
        minor_breaks = seq(-1, 1, 0.1),
        expand       = expansion(add = 0.02)
    ) +
    labs(
        x     = expression(τ[min]),
        y     = "count",
        title = "τ-min density for τ-a ≥ 0.9 and τ-b ≥ 0.9"
    ) +
    theme_minimal(base_size = 24, base_family = "LinLibertine") +
    theme(
        panel.grid.major = element_line(colour = "grey80", linewidth = 0.6),
        panel.grid.minor = element_line(colour = "grey90", linewidth = 0.45),
        axis.title       = element_text(face = "bold", size = 32),
        axis.text.y      = element_text(colour = "grey20"),
        axis.text.x      = element_text(angle = 45, colour = "grey20"),
        plot.title       = element_text(face = "bold", hjust = 0.5, size = 32),
        legend.text      = element_text(size = 24),
        legend.position  = c(0.2, 0.8)
    )

ggsave("/Users/andtsa/cse3000/andreas/final/plots/09-dist-sq.pdf", p, width = 7, height = 6)
# ggsave("/Users/andtsa/cse3000/andreas/final/plots/09-b-dist.pdf", p_b, width = 8, height = 6)

df %>%
    summarise(
        a_09 = sum(t_a >= 0.9, na.rm = TRUE),
        b_09 = sum(t_b >= 0.9, na.rm = TRUE),
        a_09_min09 = sum(t_a >= 0.9 & t_min < 0.9, na.rm = TRUE),
        a_09_min085 = sum(t_a >= 0.9 & t_min < 0.85, na.rm = TRUE),
        # a_09_min08 = sum(t_a >= 0.9 & t_min < 0.8, na.rm = TRUE),
        b_09_min09 = sum(t_b >= 0.9 & t_min < 0.9, na.rm = TRUE),
        b_09_min085 = sum(t_b >= 0.9 & t_min < 0.85, na.rm = TRUE),
        b_09_min08 = sum(t_b >= 0.9 & t_min < 0.8, na.rm = TRUE)
    )
