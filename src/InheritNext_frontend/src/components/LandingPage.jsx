import React from "react";
import {
  Box,
  Button,
  Container,
  Heading,
  Text,
  Stack,
  Icon,
  Flex,
  VStack,
} from "@chakra-ui/react";
import { FaFingerprint, FaShieldAlt, FaInfinity } from "react-icons/fa";

export const LandingPage = ({ onLogin }) => {
  return (
    <Box minH="100vh" bg="gray.50">
      {/* TODO: Will add stuff as we build */}

      {/* Navbar */}
      <Flex
        as="header"
        position="sticky"
        top={0}
        zIndex={10}
        align="center"
        justify="space-between"
        wrap="wrap"
        px={8}
        py={4}
        bg="white"
        borderBottom="1px"
        borderColor="gray.100"
        backdropFilter="blur(10px)"
        backgroundColor="rgba(255, 255, 255, 0.9)"
      >
        <Flex align="center">
          <Icon as={FaFingerprint} w={6} h={6} color="brand.600" mr={3} />
          <Heading as="h1" size="md" letterSpacing={"tight"} color="brand.900">
            InheritNext
          </Heading>
        </Flex>

        <Box>
          <Button
            size="md"
            variant="ghost"
            color="brand.600"
            onClick={onLogin}
            fontWeight="bold"
          >
            Sign In
          </Button>
          <Button ml={4} size="md" onClick={onLogin} boxShadow="md">
            Get Started
          </Button>
        </Box>
      </Flex>

      {/* Hero Section */}
      <Container maxW={"5xl"} pt={20} pb={24}>
        <Stack
          as={Box}
          textAlign={"center"}
          spacing={{ base: 8, md: 12 }}
          py={{ base: 10, md: 20 }}
        >
          <Heading
            fontWeight={800}
            fontSize={{ base: "4xl", sm: "5xl", md: "7xl" }}
            lineHeight={"110%"}
            color="gray.900"
          >
            Your Digital Legacy <br />
            <Text
              as={"span"}
              color={"brand.500"}
              css={{
                textDecoration: "underline",
                textDecorationColor: "#B2DFDB",
              }}
            >
              Secured on Chain
            </Text>
          </Heading>
          <Text
            color={"gray.600"}
            fontSize={{ base: "lg", sm: "xl", md: "2xl" }}
            maxW="3xl"
            mx="auto"
            lineHeight="1.8"
          >
            Ensure your digital assets, memories, and critical data are passed
            on securely. Decentralized, immutable, and accessible only to those
            you trust.
          </Text>
          <Stack
            direction={{ base: "column", sm: "row" }}
            spacing={4}
            align={"center"}
            justify={"center"}
            mt={8}
          >
            <Button
              rounded={"full"}
              size="lg"
              px={10}
              h={16}
              fontSize={"xl"}
              onClick={onLogin}
              leftIcon={<FaFingerprint />}
              colorScheme="brand"
            >
              Login with Internet Identity
            </Button>
          </Stack>
          <Text fontSize="sm" color="gray.500" mt={4}>
            Powered by Internet Computer • No password required
          </Text>
        </Stack>

        {/* Features Section */}
        <Box mt={24}>
          <VStack spacing={20}>
            <FeatureRow
              index={0}
              icon={FaShieldAlt}
              title="Unbreakable Security"
              text="Your legacy is protected by the Internet Computer Protocol. Data is encrypted and sharded across a global network of independent data centers, making it impossible to compromise."
            />
            <FeatureRow
              index={1}
              icon={FaInfinity}
              title="Perpetual Storage"
              text="Smart contracts act as your digital executor. They live forever on the blockchain, ensuring your instructions are carried out exactly as written, even decades from now."
            />
            <FeatureRow
              index={2}
              icon={FaFingerprint}
              title="Sovereign Identity"
              text="No usernames, no passwords, no tracking. Authenticate securely with Internet Identity using your device's biometrics. You own your data completely."
            />
          </VStack>
        </Box>
      </Container>

      <Box
        py={10}
        textAlign="center"
        borderTop="1px"
        borderColor="gray.200"
        bg="white"
      >
        <Text color="gray.500" fontSize="sm">© {new Date().getFullYear()} InheritNext. Built on Internet Computer.</Text>
      </Box>
    </Box>
  );
};

const FeatureRow = ({ title, text, icon, index }) => {
  const isEven = index % 2 === 0;
  return (
    <Flex
      direction={{ base: "column", md: isEven ? "row" : "row-reverse" }}
      align="center"
      justify="space-between"
      gap={{ base: 8, md: 16 }}
    >
      <Box flex={1}>
        <Flex
          w={16}
          h={16}
          align={"center"}
          justify={"center"}
          rounded={"full"}
          bg={"brand.50"}
          color={"brand.600"}
          mb={6}
        >
          <Icon as={icon} w={8} h={8} />
        </Flex>
        <Heading size="lg" fontWeight="bold" color="brand.900" mb={4}>
          {title}
        </Heading>
        <Text color={"gray.600"} fontSize="lg" lineHeight="1.8">
          {text}
        </Text>
      </Box>
      <Box
        flex={1}
        h="300px"
        w="full"
        bg="gray.50"
        rounded="2xl"
        position="relative"
        overflow="hidden"
        boxShadow="lg"
        border="1px solid"
        borderColor="gray.100"
      >
        {/* Feature illustration */}
        <Flex 
          h="full" 
          align="center" 
          justify="center" 
          bgGradient={`linear(to-br, ${isEven ? 'brand.50' : 'orange.50'}, white)`}
        >
           <Icon 
             as={icon} 
             w={40} 
             h={40} 
             color={isEven ? 'brand.200' : 'orange.200'} 
             filter="drop-shadow(0px 2px 4px rgba(0,0,0,0.1))"
           />
        </Flex>
      </Box>
    </Flex>
  );
};

