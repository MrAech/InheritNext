import React, { useState } from "react";
import {
  Box,
  Button,
  FormControl,
  FormLabel,
  Input,
  VStack,
  Heading,
  Text,
  Container,
  Card,
  CardBody,
  Divider,
  Progress,
  Checkbox,
  Stack,
  HStack,
  useToast,
  Alert,
  AlertIcon,
  AlertTitle,
  AlertDescription,
  Step,
  StepDescription,
  StepIcon,
  StepIndicator,
  StepNumber,
  StepSeparator,
  StepStatus,
  StepTitle,
  Stepper,
  useSteps,
} from "@chakra-ui/react";

const STEPS = [
  { title: "Identity", description: "Who are you?" },
  { title: "Purpose", description: "Risks" },
  { title: "Review", description: "Confirm" },
];

export const Onboarding = ({ onSubmit, isLoading }) => {
  const { activeStep, setActiveStep } = useSteps({
    index: 0,
    count: STEPS.length,
  });
  
  const [formData, setFormData] = useState({
    firstName: "",
    lastName: "",
    purposeAcknowledged: false,
    recoveryPrincipals: "",
  });

  const toast = useToast();

  const handleInputChange = (e) => {
    const { name, value, type, checked } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: type === "checkbox" ? checked : value,
    }));
  };

  const handleNext = () => {
    if (activeStep === 0) {
      if (!formData.firstName.trim() || !formData.lastName.trim()) {
        toast({
          title: "Required",
          description: "Please enter your first and last name.",
          status: "error",
          duration: 3000,
        });
        return;
      }
    }
    if (activeStep === 1) {
      if (!formData.purposeAcknowledged) {
        toast({
          title: "Acknowledgement Required",
          description: "You must acknowledge the purpose of this application.",
          status: "error",
          duration: 3000,
        });
        return;
      }
    }

    if (activeStep < STEPS.length - 1) {
      setActiveStep(activeStep + 1);
    }
  };

  const handleBack = () => {
    if (activeStep > 0) {
      setActiveStep(activeStep - 1);
    }
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(formData);
  };

  const renderStepContent = () => {
    switch (activeStep) {
      case 0: // Identity
        return (
          <VStack spacing={6} align="stretch">
            <Box textAlign="center" mb={4}>
                <Heading size="md" color="brand.700" mb={2}>
                Identity Setup
                </Heading>
                <Text color="gray.500" fontSize="sm">Please provide your real name for your heirs to recognize.</Text>
            </Box>
            <FormControl id="firstName" isRequired>
              <FormLabel fontWeight="bold" color="gray.700">
                First Name
              </FormLabel>
              <Input
                name="firstName"
                value={formData.firstName}
                onChange={handleInputChange}
                placeholder="e.g. Satoshi"
                size="lg"
                focusBorderColor="brand.500"
              />
            </FormControl>
            <FormControl id="lastName" isRequired>
              <FormLabel fontWeight="bold" color="gray.700">
                Last Name
              </FormLabel>
              <Input
                name="lastName"
                value={formData.lastName}
                onChange={handleInputChange}
                placeholder="e.g. Nakamoto"
                size="lg"
                focusBorderColor="brand.500"
              />
            </FormControl>
          </VStack>
        );

      case 1: // Purpose
        return (
          <VStack spacing={6} textAlign="left" align="stretch">
             <Box textAlign="center" mb={4}>
                <Heading size="md" color="brand.700" mb={2}>
                Purpose & Responsibility
                </Heading>
            </Box>
            <Alert
              status="warning"
              variant="subtle"
              flexDirection="column"
              alignItems="center"
              justifyContent="center"
              textAlign="center"
              borderRadius="lg"
              p={6}
              border="1px solid"
              borderColor="orange.200"
            >
              <AlertIcon boxSize="40px" mr={0} />
              <AlertTitle mt={4} mb={1} fontSize="lg" color="orange.800">
                Irreversible Action
              </AlertTitle>
              <AlertDescription maxWidth="sm" color="orange.700">
                This application acts as a digital <b>Dead Man's Switch</b>. If you
                fail to check in, your stored secrets <b>will be released</b> to
                your designated heirs. This process cannot be undone once
                triggered.
              </AlertDescription>
            </Alert>
            <Box p={4} bg="gray.50" borderRadius="md">
                <Checkbox
                name="purposeAcknowledged"
                isChecked={formData.purposeAcknowledged}
                onChange={handleInputChange}
                colorScheme="brand"
                size="lg"
                spacing={4}
                alignItems="start"
                >
                <Text fontSize="sm" fontWeight="bold" pt={1}>I understand that this app creates an automated release mechanism.</Text>
                </Checkbox>
            </Box>
          </VStack>
        );

      case 2: // Review
        return (
          <VStack spacing={6} align="stretch">
             <Box textAlign="center" mb={4}>
                <Heading size="md" color="brand.700" mb={2}>
                Review & Confirm
                </Heading>
            </Box>
            <Card variant="outline" bg="gray.50">
                <CardBody>
                    <Stack spacing={4}>
                        <HStack justify="space-between">
                        <Text fontWeight="bold" color="gray.600">Name</Text>
                        <Text fontWeight="medium">
                            {formData.firstName} {formData.lastName}
                        </Text>
                        </HStack>
                        <Divider />
                        <HStack justify="space-between">
                             <Text fontWeight="bold" color="gray.600">Acknowledgement</Text>
                             <Text color="green.600" fontWeight="bold">Signed</Text>
                        </HStack>
                    </Stack>
                </CardBody>
            </Card>
             <Text fontSize="xs" color="gray.500" textAlign="center">
                  By creating an account, you agree to the Terms of Service.
            </Text>
          </VStack>
        );
      default:
        return null;
    }
  };

  return (
    <Box minH="100vh" bg="gray.50" py={20}>
      <Container maxW="2xl">
        <VStack spacing={8} align="stretch">
          <Box textAlign="center" mb={6}>
            <Heading size="xl" color="brand.900" letterSpacing="tight">
              Create Your Legacy
            </Heading>
            <Text color="gray.500" mt={3} fontSize="lg">
              Set up your secure digital vault in minutes.
            </Text>
          </Box>

            <Box mb={8}>
                <Stepper index={activeStep} colorScheme="brand">
                    {STEPS.map((step, index) => (
                    <Step key={index}>
                        <StepIndicator>
                        <StepStatus
                            complete={<StepIcon />}
                            incomplete={<StepNumber />}
                            active={<StepNumber />}
                        />
                        </StepIndicator>

                        <Box flexShrink='0'>
                        <StepTitle>{step.title}</StepTitle>
                        <StepDescription>{step.description}</StepDescription>
                        </Box>

                        <StepSeparator />
                    </Step>
                    ))}
                </Stepper>
            </Box>

          <Card boxShadow="xl" bg="white" borderRadius="2xl" border="1px" borderColor="gray.100">
            <CardBody p={8}>
              <form onSubmit={handleSubmit}>
                <VStack spacing={8} minH="350px" justify="space-between">
                  <Box w="full">
                    {renderStepContent()}
                  </Box>

                  <HStack justify="space-between" w="full" pt={4}>
                    <Button
                      onClick={handleBack}
                      isDisabled={activeStep === 0 || isLoading}
                      variant="ghost"
                      size="lg"
                    >
                      Back
                    </Button>

                    {activeStep === STEPS.length - 1 ? (
                      <Button
                        type="submit"
                        size="lg"
                        isLoading={isLoading}
                        loadingText="Creating Account..."
                        colorScheme="brand"
                        px={8}
                      >
                        Create Account
                      </Button>
                    ) : (
                      <Button onClick={handleNext} size="lg" colorScheme="brand" px={8}>
                        Next
                      </Button>
                    )}
                  </HStack>
                </VStack>
              </form>
            </CardBody>
          </Card>
        </VStack>
      </Container>
    </Box>
  );
};

