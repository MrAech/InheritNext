import { extendTheme } from "@chakra-ui/react";

const theme = extendTheme({
  config: {
    initialColorMode: "light",
    useSystemColorMode: false,
  },

  colors: {
    brand: {
      50: "#E0F2F1",
      100: "#B2DFDB",
      200: "#80CBC4",
      300: "#4DB6AC",
      400: "#26A69A",
      500: "#009688", // Primary Teal
      600: "#00897B",
      700: "#00796B",
      800: "#00695C",
      900: "#004D40",
    },
    gray: {
      50: "#F9FAFB",
      100: "#F3F4F6",
      200: "#E5E7EB",
      300: "#D1D5DB",
      400: "#9CA3AF",
      500: "#6B7280",
      600: "#4B5563",
      700: "#374151",
      800: "#1F2937",
      900: "#111827",
    },
    accent: {
      500: "#F59E0B", // Amber for highlights
      600: "#D97706",
    },
  },

  fonts: {
    heading: `'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`,
    body: `'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`,
  },

  components: {
    Button: {
      baseStyle: {
        borderRadius: "lg", // Softer corners
        fontWeight: "600",
        transition: "all 0.2s cubic-bezier(.08,.52,.52,1)",
      },
      variants: {
        solid: {
          bg: "brand.600",
          color: "white",
          _hover: {
            bg: "brand.700",
            transform: "translateY(-1px)",
            boxShadow: "md",
          },
          _active: {
            bg: "brand.800",
            transform: "translateY(0)",
          },
        },
        outline: {
          borderColor: "brand.600",
          color: "brand.600",
          _hover: {
            bg: "brand.50",
          },
        },
        ghost: {
          color: "brand.600",
          _hover: {
            bg: "brand.50",
          },
        },
      },
      defaultProps: {
        size: "lg", // Default to larger, more clickable buttons
        variant: "solid",
      },
    },

    Input: {
      baseStyle: {
        field: {
          borderRadius: "lg",
        },
      },
      variants: {
        filled: {
          field: {
            bg: "white",
            border: "1px solid",
            borderColor: "gray.200",
            _hover: { bg: "gray.50" },
            _focus: {
              bg: "white",
              borderColor: "brand.500",
              boxShadow: "0 0 0 1px #009688",
            },
          },
        },
      },
      defaultProps: {
        variant: "filled",
        size: "lg",
      },
    },

    Card: {
      baseStyle: {
        container: {
          borderRadius: "xl",
          boxShadow: "lg", // Softer, spread out shadow
          bg: "white",
          border: "1px solid",
          borderColor: "gray.100",
        },
      },
    },

    Heading: {
      baseStyle: {
        fontWeight: "700",
        letterSpacing: "-0.02em",
        color: "gray.800",
      },
    },

    Text: {
      baseStyle: {
        color: "gray.600",
        lineHeight: "1.6",
      },
    },
  },

  styles: {
    global: {
      body: {
        bg: "gray.50",
        color: "gray.800",
        fontFamily: "body",
      },
    },
  },
});

export default theme;
