import { Avatar, Badge, Box, Button, Card, CardBody, Container, Divider, Flex, Heading, HStack, Icon, Menu, MenuButton, MenuItem, MenuList, SimpleGrid, Spinner, Stat, StatHelpText, StatLabel, StatNumber, Tab, TabList, TabPanel, TabPanels, Tabs, Text, Tooltip, useToast, VStack } from "@chakra-ui/react"
import { useEffect, useState } from "react";
import { calculateDmsStatus } from "../icp-utils";
import { FaChevronDown, FaClock, FaCog, FaFileContract, FaHeartbeat, FaPlus, FaQuestionCircle, FaSignOutAlt, FaWallet } from "react-icons/fa";



const StatCard = ({ label, number, helpText, icon, colorScheme = "brand", onClick, tooltip }) => {
    const content = (
        <Card
            height="100%"
            cursor={onClick ? "pointer" : "default"}
            _hover={onClick ? { borderColor: "brand.300", boxShadow: "lg" } : {}}
            transition="all 0.2s"
            onClick={onClick}
        >
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

    if (tooltip) {
        return <Tooltip label={tooltip}>{content}</Tooltip>
    }
    return content;
};

export const Dashboard = ({ profile, actor,  onLogout }) => {
    const [vault, setVault] = useState(null);
    const [loading, setLoading] = useState(true);
    const [checkingIn, setCheckingIn] = useState(false);
    const [creatingVault, setCreatingVault] = useState(false);
    const toast = useToast();

    const fullName = profile ?
        `${profile.first_name} ${profile.last_name}`
        : "User";

    useEffect(() => {
        if (actor) {
            loadAllData();
        }
    }, [actor]);

    const loadAllData = async () => {
        setLoading(true);
        await Promise.all([loadVault()]);
        setLoading(false);
    };

    const loadVault = async () => {
        try {
            const res = await actor.get_my_vault();
            if (res.Ok) {
                setVault(res.Ok);
            } else {
                if (res.Err && res.Err.includes("No vault")) {
                    setVault(null);

                } else {
                    console.error("Failed to Load Vault", res.Err);
                }
            }
        } catch (e) {
            console.error("Error loading vault:", e);
        }
    }

    const handleCreateVault = async () => {
        try {
            setCreatingVault(true);
            const res = await actor.create_vault();
            if (res.Err) {
                throw new Error(res.Err);
            }
            toast({
                title: "Vault Activated",
                description: "Your secure vault has been created. Now add assets and beneficiaries!",
                status: "success",
                duration: 4000,
                isClosable: true
            });
            await loadVault();
        } catch (e) {
            toast({
                title: "Activation Failed",
                description: e.message || "Could not create vault.",
                status: "error",
                isClosable: true
            });
        } finally {
            setCreatingVault(false);
        }
    };

    const handleCheckIn = async () => {
        try {
            setCheckingIn(true);
            const res = await actor.heartbeat();
            if (res.Err) {
                throw new Error(res.Err);

            }

            toast({
                title: "Check-in Successful",
                description: "Your timer has been reset. See you next time!",
                status: "success",
                duration: 3000,
                isClosable: true
            });


            await loadVault();
        } catch (e) {
            toast({
                title: "Check-in Failed",
                description: e.message || "Could not send heartbeat.",
                status: "error",
                isClosable: true
            });
        } finally {
            setCheckingIn(false);
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
                    alignItems="center"
                    justifyContent="space-between"
                    maxW="7xl"
                    mx="auto"
                >
                    <Flex
                        alignItems="center" gap={3}
                    >
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


                    <Flex alignItems="center" gap={6}>
                        <Tooltip label="Help and Document">
                            <Button variant="ghost" size="sm" borderRadius="full">
                                <Icon as={FaQuestionCircle} color="gray.400" />
                            </Button>
                        </Tooltip>
                        <Menu>
                            <MenuButton
                                as={Button}
                                rounded="full"
                                variant="ghost"
                                cursor="pointer"
                                minW={0}
                                p={1}
                            >
                                <Flex align="center" gap={3}>
                                    <Avatar size="sm" name={fullName} bg="brand.500" color="white" />
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
                                <MenuItem borderRadius="md" isDisabled={!vault}>
                                    <HStack>
                                        <Icon as={FaCog} />
                                        <Text>Check In Settings</Text>
                                    </HStack>
                                </MenuItem>
                                <Divider my={2} />
                                <MenuItem icon={<FaSignOutAlt />} onClick={onLogout} color="red.500" borderRadius="md">
                                    Sign Out
                                </MenuItem>
                            </MenuList>
                        </Menu>
                    </Flex>
                </Flex>
            </Box>

            <Container maxW="7xl" py={8}>
                {
                    loading ? (
                        <Flex justify="center" align="center" minH="400px">
                            <Spinner size="xl" color="brand.500" thickness="4px" />
                        </Flex>
                    ) : (
                        <>
                            {/* Vault Activation and Status Banner */}
                            {
                                !vault ? (
                                    <Box mb={8}>
                                        <Card variant="outline" bg="blue.50" borderColor="blue.100" overflow="hidden">
                                            <CardBody p={{ base: 6, md: 8 }}>
                                                <Flex direction={{ base: "column", md: "row" }} align="center" justify="space-between" gap={6}>
                                                    <Flex gap={5} align="start">
                                                        <Box p={4} bg="white" borderRadius="xl" boxShadow="sm" color="blue.500">
                                                            <Icon as={FaWallet} w={6} h={6} />
                                                        </Box>
                                                        <Box>
                                                            <Heading size="md" mb={2} color="blue.800">Welcome! Let's Set Up Your Vault</Heading>
                                                            <Text color="blue.700" maxW="xl">
                                                                Your vault is where you store digital assets and designate who should receive them.
                                                                It's like a digital safety deposit box with automatic inheritance.
                                                            </Text>
                                                        </Box>
                                                    </Flex>
                                                    <Button
                                                        colorScheme="blue"
                                                        size="lg"
                                                        leftIcon={<FaPlus />}
                                                        onClick={handleCreateVault}
                                                        isLoading={creatingVault}
                                                        loadingText="Creating..."
                                                        boxShadow="md"
                                                        flexShrink={0}
                                                    >
                                                        Create My Vault
                                                    </Button>
                                                </Flex>
                                            </CardBody>
                                        </Card>
                                    </Box>
                                ) : (
                                    <Box mb={8}>
                                        <Card
                                            bg={dmsInfo?.isOverdue ? "red.50" : "green.50"}
                                            borderColor={dmsInfo?.isOverdue ? "red.100" : "green.100"}
                                            variant="outline"
                                        >
                                            <CardBody p={{ base: 6, md: 8 }}>
                                                <Flex direction={{ base: "column", md: "row" }} align="center" justify="space-between" gap={6}>
                                                    <Flex gap={5} align="start">
                                                        <Box p={4} bg="white" borderRadius="xl" boxShadow="sm" color={dmsInfo?.isOverdue ? "red.500" : "green.500"}>
                                                            <Icon as={FaHeartbeat} w={6} h={6} />
                                                        </Box>
                                                        <Box>
                                                            <HStack mb={2}>
                                                                <Heading size="md" color={dmsInfo?.isOverdue ? "red.800" : "green.800"}>
                                                                    {dmsInfo?.isOverdue ? "Check In Overdue!" : "You are all Set"}
                                                                </Heading>
                                                                <Tooltip label="Click to Chnage check-in schedule">
                                                                    <Button size="sm" variant="ghost" >
                                                                        <Icon as={FaCog} />
                                                                    </Button>
                                                                </Tooltip>
                                                            </HStack>
                                                            {vault?.status && (
                                                                <Text fontSize="sm" color="gray.600" mb={2}>
                                                                    <b>Vault Status: </b>{vault.status}
                                                                </Text>
                                                            )}
                                                            <Text color={dmsInfo?.isOverdue ? "red.700" : "green.700"} maxW="xl">
                                                                {dmsInfo?.isOverdue
                                                                    ? "You've missed your check-in deadline. Your assets may be transferred to beneficiaries soon."
                                                                    : `Everything looks good! Your next check-in is due by ${dmsInfo?.nextDueDate}. You have ${dmsInfo?.daysRemaining} days remaining.`}
                                                            </Text>
                                                        </Box>
                                                    </Flex>
                                                    <Button
                                                        colorScheme={dmsInfo?.isOverdue ? "red" : "green"}
                                                        size="lg"
                                                        leftIcon={<FaClock />}

                                                        onClick={handleCheckIn}
                                                        isLoading={checkingIn}
                                                        loadingText="Checking you in..."
                                                        boxShadow="md"
                                                        flexShrink={0}
                                                    >
                                                        Check In Now
                                                    </Button>
                                                </Flex>
                                            </CardBody>
                                        </Card>
                                    </Box>
                                )}

                            <SimpleGrid columns={{ base: 1, sm: 2, md: 4 }} spacing={6} mb={8} mt={6}>
                                <StatCard
                                    label="Next Check-in"
                                    number={dmsInfo ? `${dmsInfo?.daysRemaining}d` : "-"}
                                    helpText={dmsInfo ? dmsInfo.nextDueDate : "Vault not active"}
                                    icon={FaClock}
                                    colorScheme={dmsInfo?.daysRemaining < 3 ? "red" : dmsInfo?.daysRemaining < 7 ? "orange" : "brand"}
                                />

                            </SimpleGrid>

                            {
                                vault && (
                                    <Card>
                                        <Tabs
                                            variant="enclosed"
                                            colorScheme="brand"
                                        >
                                            <TabList px={4} pt={4}>
                                                <Tab fontWeight="semibold" _selected={{ color: "brand.600", borderColor: "brand.600" }}>
                                                    <HStack spacing={2}>
                                                        <Icon as={FaWallet} />
                                                        <Text>My Assets</Text>
                                                        <Badge colorScheme="brand" borderRadius="full">{0}</Badge>
                                                    </HStack>
                                                </Tab>
                                            </TabList>
                                            <TabPanels>
                                                <TabPanel p={6}>
                                                    <Flex justify="space-between" align="center" mb={6}>
                                                        <Box>
                                                            <Heading size="md" color="gray.800">Your Assets</Heading>
                                                            <Text color="gray.500" fontSize="sm" mt={1}>
                                                                Assets stored here will be transferred to your beneficiaries
                                                            </Text>
                                                        </Box>
                                                    </Flex>
                                                </TabPanel>
                                            </TabPanels>
                                        </Tabs>
                                    </Card>
                                )}

                            {
                                !vault && (
                                    <Card minH="400px">
                                        <CardBody
                                            display="flex"
                                            flexDirection="column"
                                            alignItems="center"
                                            justifyContent="center"
                                            py={16}
                                        // 16 !!!!!!!!!!
                                        >
                                            <VStack spacing={6} maxW="lg" textAlign="center">
                                                <Box p={6} bg="gray.50" borderRadius="full">
                                                    <Icon as={FaFileContract} w={16} h={16} color="gray.300" />
                                                </Box>
                                                <Heading size="lg" color="gray.600">
                                                    Your Digital Legacy Starts Here
                                                </Heading>
                                                <Text color="gray.500" fontSize="lg">
                                                    Create your vault to start securing your digital assets.
                                                    Add cryptocurrency, NFTs, or even smart contracts — then
                                                    choose who should inherit them.
                                                </Text>
                                                <VStack spacing={3} w="full" maxW="sm" pt={4}>
                                                    <Button
                                                        size="lg"
                                                        w="full"
                                                        colorScheme="brand"
                                                        leftIcon={<FaPlus />}
                                                        onClick={handleCreateVault}
                                                        isLoading={creatingVault}
                                                    >
                                                        Create My Vault
                                                    </Button>
                                                </VStack>
                                            </VStack>
                                        </CardBody>
                                    </Card>
                                )
                            }
                        </>
                    )
                }
            </Container>
        </Box >
    );
};

