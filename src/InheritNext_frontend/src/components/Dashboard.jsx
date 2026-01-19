import React, { useEffect, useState } from "react";
import {
  Box,
  Flex,
  Heading,
  Text,
  Button,
  Icon,
  Avatar,
  Menu,
  MenuButton,
  MenuList,
  MenuItem,
  Stat,
  StatLabel,
  StatNumber,
  StatHelpText,
  Container,
  SimpleGrid,
  Card,
  CardHeader,
  CardBody,
  Divider,
  useToast,
  Spinner,
  Alert,
  AlertTitle,
  AlertDescription,
  HStack,
  VStack,
} from "@chakra-ui/react";
import {
  FaPlus,
  FaUserFriends,
  FaFileContract,
  FaBell,
  FaChevronDown,
  FaSignOutAlt,
  FaWallet,
  FaHeartbeat,
  FaClock,
} from "react-icons/fa";
import { calculateDmsStatus } from "../icp-utils";

const StatCard = ({ label, number, helpText, icon, colorScheme = "brand" }) => (
  <Card height="100%">
    <CardBody>
      <Flex justifyContent="space-between" alignItems="start">
        <Stat>
          <StatLabel color="gray.500" fontWeight="semibold" fontSize="sm">
            {label}
          </StatLabel>
          <StatNumber
            fontSize="3xl"
            fontWeight="bold"
            color={`${colorScheme}.600`}
            my={1}
          >
            {number}
          </StatNumber>
          <StatHelpText color="gray.400" fontSize="xs">{helpText}</StatHelpText>
        </Stat>
        <Box
          p={3}
          bg={`${colorScheme}.50`}
          borderRadius="lg"
          color={`${colorScheme}.500`}
        >
          <Icon as={icon} w={5} h={5} />
        </Box>
      </Flex>
    </CardBody>
  </Card>
);

export const Dashboard = ({ profile, actor, onLogout }) => {
  const [vault, setVault] = useState(null);
  const [loading, setLoading] = useState(true);
  const [checkingIn, setCheckingIn] = useState(false);
  const [creatingVault, setCreatingVault] = useState(false);
  const toast = useToast();

  const fullName = profile
    ? `${profile.first_name} ${profile.last_name}`
    : "User";

  useEffect(() => {
    if (actor) {
      loadVault();
    }
  }, [actor]);

  const loadVault = async () => {
    try {
      setLoading(true);
      const res = await actor.get_my_vault();
      if (res.Ok) {
        setVault(res.Ok);
      } else {
        if (res.Err && res.Err.includes("No vault found")) {
          console.log("Setting vault to null");
          setVault(null);
        } else {
          console.error("Failed to load vault:", res.Err);
          toast({
            title: "Error",
            description: "Could not load vault data.",
            status: "error",
          });
        }
      }
    } catch (error) {
      console.error("Error loading vault:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateVault = async () => {
    try {
      setCreatingVault(true);
      const res = await actor.create_vault();
      if (res.Err) {
        throw new Error(res.Err);
      }
      toast({
        title: "Vault Activated",
        description: "Your secure vault has been created.",
        status: "success",
        duration: 3000,
      });
      await loadVault();
    } catch (error) {
      toast({
        title: "Activation Failed",
        description: error.message || "Could not create vault.",
        status: "error",
      });
    } finally {
      setCreatingVault(false);
    }
  };

  const handleCheckIn = () => {
    // TODO: Wire up to backend
    toast({
      title: "Check-in Successful",
      description: "Your timer has been reset.",
      status: "success",
      duration: 3000,
    });

    if (vault) {
      setVault({
        ...vault,
        dms: {
          ...vault.dms,
          last_heartbeat: BigInt(Date.now()) * 1000000n,
          pending_since: [],
        },
      });
    }
  };

  const dmsInfo = calculateDmsStatus(vault);

  return (
    <Box minH="100vh" bg="gray.50">
      <Box
        bg="white"
        px={6}
        borderBottom="1px"
        borderColor="gray.200"
        position="sticky"
        top={0}
        zIndex={10}
        boxShadow="sm"
      >
        <Flex
          h={20}
          alignItems={"center"}
          justifyContent={"space-between"}
          maxW="7xl"
          mx="auto"
        >
          <Flex alignItems="center" gap={3}>
            <Box
              w={10}
              h={10}
              bg="brand.600"
              borderRadius="lg"
              display="flex"
              alignItems="center"
              justifyContent="center"
              boxShadow="md"
            >
              <Icon as={FaFileContract} color="white" w={5} h={5} />
            </Box>
            <Heading size="md" color="gray.800" letterSpacing="tight">
              InheritNext
            </Heading>
          </Flex>

          <Flex alignItems={"center"} gap={6}>
            <Button variant="ghost" size="sm" borderRadius="full">
              <Icon as={FaBell} color="gray.400" />
            </Button>
            <Menu>
              <MenuButton
                as={Button}
                rounded={"full"}
                variant={"ghost"}
                cursor={"pointer"}
                minW={0}
                p={1}
              >
                <Flex align="center" gap={3}>
                  <Avatar size={"sm"} name={fullName} bg="brand.500" color="white" />
                  <Box display={{ base: "none", md: "block" }} textAlign="left">
                    <Text fontSize="sm" fontWeight="bold" color="gray.700">
                      {fullName}
                    </Text>
                  </Box>
                  <Icon
                    as={FaChevronDown}
                    color="gray.400"
                    fontSize="xs"
                    display={{ base: "none", md: "block" }}
                  />
                </Flex>
              </MenuButton>
              <MenuList boxShadow="lg" border="none" p={2}>
                <MenuItem borderRadius="md">Profile</MenuItem>
                <MenuItem borderRadius="md">Settings</MenuItem>
                <Divider my={2} />
                <MenuItem icon={<FaSignOutAlt />} onClick={onLogout} color="red.500" borderRadius="md">
                  Sign Out
                </MenuItem>
              </MenuList>
            </Menu>
          </Flex>
        </Flex>
      </Box>

      <Container maxW="7xl" py={12}>
        {loading ? (
          <Flex justify="center" align="center" minH="400px">
            <Spinner size="xl" color="brand.500" thickness="4px" />
          </Flex>
        ) : (
          <>
            {!vault ? (
              <Box mb={10}>
                <Card variant="outline" bg="blue.50" borderColor="blue.100" overflow="hidden">
                    <CardBody p={{base: 6, md: 8}}>
                        <Flex direction={{ base: "column", md: "row" }} align="center" justify="space-between" gap={6}>
                            <Flex gap={5} align="start">
                                <Box p={4} bg="white" borderRadius="xl" boxShadow="sm" color="blue.500">
                                    <Icon as={FaWallet} w={6} h={6} />
                                </Box>
                                <Box>
                                    <Heading size="md" mb={2} color="blue.800">Activate Your Vault</Heading>
                                    <Text color="blue.700" maxW="xl">Your account is ready. Initialize your secure vault to start storing assets and designating heirs. This creates your personal smart contract canister.</Text>
                                </Box>
                            </Flex>
                             <Button
                                colorScheme="blue"
                                size="lg"
                                leftIcon={<FaPlus />}
                                onClick={handleCreateVault}
                                isLoading={creatingVault}
                                loadingText="Activating..."
                                boxShadow="md"
                                flexShrink={0}
                              >
                                Activate Vault
                              </Button>
                        </Flex>
                    </CardBody>
                </Card>
              </Box>
            ) : (
              dmsInfo && (
                <Box mb={10}>
                  <Card 
                    bg={dmsInfo.isOverdue ? "red.50" : "green.50"} 
                    borderColor={dmsInfo.isOverdue ? "red.100" : "green.100"}
                    variant="outline"
                  >
                     <CardBody p={{base: 6, md: 8}}>
                        <Flex direction={{ base: "column", md: "row" }} align="center" justify="space-between" gap={6}>
                            <Flex gap={5} align="start">
                                <Box p={4} bg="white" borderRadius="xl" boxShadow="sm" color={dmsInfo.isOverdue ? "red.500" : "green.500"}>
                                    <Icon as={FaHeartbeat} w={6} h={6} />
                                </Box>
                                <Box>
                                    <Heading size="md" mb={2} color={dmsInfo.isOverdue ? "red.800" : "green.800"}>
                                         {dmsInfo.isOverdue
                                            ? "Check-in Overdue!"
                                            : "System Status: Secure"}
                                    </Heading>
                                    <Text color={dmsInfo.isOverdue ? "red.700" : "green.700"} maxW="xl">
                                          {dmsInfo.isOverdue
                                            ? "You have missed your check-in deadline. Release protocol may be initiating."
                                            : `You are safe. Next check-in is due by ${dmsInfo.nextDueDate}.`}
                                    </Text>
                                </Box>
                            </Flex>
                            <Button
                              colorScheme={dmsInfo.isOverdue ? "red" : "green"}
                              size="lg"
                              leftIcon={<FaClock />}
                              onClick={handleCheckIn}
                              isLoading={checkingIn}
                              loadingText="Checking In..."
                              boxShadow="md"
                              flexShrink={0}
                            >
                              Check In
                            </Button>
                        </Flex>
                     </CardBody>
                  </Card>
                </Box>
              )
            )}{" "}
            <Flex justifyContent="space-between" alignItems="center" mb={8}>
              <Box>
                <Heading size="lg" mb={2} color="gray.800">
                  Overview
                </Heading>
                <Text color="gray.500" fontSize="lg">
                  Welcome back! Here is the status of your vault.
                </Text>
              </Box>
              <Button
                leftIcon={<FaPlus />}
                colorScheme="brand"
                size="md"
                isDisabled={true} // Disabled until backend implementation
                onClick={() =>
                  toast({
                    title: "Feature Coming Soon",
                    description:
                      "Asset management is currently under development.",
                    status: "info",
                  })
                }
              >
                Add Asset
              </Button>
            </Flex>
            <SimpleGrid columns={{ base: 1, md: 3 }} spacing={8} mb={10}>
              <StatCard
                label="Secure Assets"
                number="0"
                helpText="Ready to be added"
                icon={FaWallet}
              />
              <StatCard
                label="Beneficiaries"
                number="0"
                helpText="Pending designation"
                icon={FaUserFriends}
              />
              <StatCard
                label="Check-in Status"
                number={dmsInfo ? `${dmsInfo.daysRemaining} Days` : "-"}
                helpText={
                  dmsInfo ? "Remaining until next check-in" : "Vault inactive"
                }
                icon={FaClock}
                colorScheme={dmsInfo?.daysRemaining < 3 ? "red" : "brand"}
              />
            </SimpleGrid>
            <Card minH="400px">
              <CardHeader pb={0}>
                <Heading size="md" color="gray.700">Recent Activity</Heading>
              </CardHeader>
              <CardBody
                display="flex"
                flexDirection="column"
                alignItems="center"
                justifyContent="center"
                py={10}
              >
                <VStack spacing={4} maxW="md" textAlign="center">
                    <Box p={5} bg="gray.50" borderRadius="full">
                        <Icon as={FaFileContract} w={10} h={10} color="gray.300" />
                    </Box>
                    <Heading size="sm" color="gray.500">
                    No activity yet
                    </Heading>
                    <Text color="gray.400">
                    Start by adding your first digital asset or designating a
                    beneficiary to ensure your legacy is secure.
                    </Text>
                     <Button
                        variant="outline"
                        mt={4}
                        onClick={!vault ? handleCreateVault : undefined}
                        isLoading={creatingVault && !vault}
                        isDisabled={vault !== null} // Disabled "Configure Vault" if vault exists
                        >
                        {!vault ? "Activate Vault" : "Configure Vault"}
                    </Button>
                </VStack>
              </CardBody>
            </Card>
          </>
        )}
      </Container>
    </Box>
  );
};
