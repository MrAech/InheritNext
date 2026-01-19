import { useState, useEffect } from "react";
import { AuthClient } from "@icp-sdk/auth/client";
import { ChakraProvider, Spinner, Flex, useToast } from "@chakra-ui/react";
import { createBackendActor } from "./icp-utils";
import theme from "./theme";
import { LandingPage } from "./components/LandingPage";
import { Onboarding } from "./components/Onboarding";
import { Dashboard } from "./components/Dashboard";

function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isRegistered, setIsRegistered] = useState(false);
  const [actor, setActor] = useState(null);
  const [loading, setLoading] = useState(true);
  const [profile, setProfile] = useState(null);
  const [authClient, setAuthClient] = useState(null);
  const toast = useToast();

  useEffect(() => {
    initAuth();
  }, []);

  async function initAuth() {
    try {
      const client = await AuthClient.create();
      setAuthClient(client);

      if (await client.isAuthenticated()) {
        await handleAuthenticated(client);
      } else {
        setLoading(false);
      }
    } catch (e) {
      console.error("Auth init error:", e);
      setLoading(false);
    }
  }

  async function handleAuthenticated(client) {
    const identity = client.getIdentity();
    try {
      const newActor = await createBackendActor(identity);
      setActor(newActor);
      setIsAuthenticated(true);

      const registered = await newActor.is_registered();
      setIsRegistered(registered);
      if (registered) {
        const userProfile = await newActor.gt_profile();
        if (userProfile.Ok) {
          setProfile(userProfile.Ok);
        }
      }
    } catch (e) {
      console.error("Authenticated setup failed", e);
    }
    setLoading(false);
  }

  async function login() {
    const client = authClient || (await AuthClient.create());
    if (!authClient) setAuthClient(client);

    const iiCanisterId = process.env.CANISTER_ID_INTERNET_IDENTITY;
    await client.login({
      identityProvider:
        process.env.DFX_NETWORK === "ic"
          ? "https://identity.ic0.app"
          : `http://${iiCanisterId}.localhost:4943`,
      onSuccess: () => handleAuthenticated(client),
    });
  }

  async function logout() {
    if (authClient) {
      await authClient.logout();
    }
    setIsAuthenticated(false);
    setIsRegistered(false);
    setActor(null);
    setProfile(null);
    setLoading(false);
  }

  async function handleRegister(formData) {
    const { firstName, lastName } = formData;

    if (actor) {
      setLoading(true);
      try {
        const regResult = await actor.register_user(firstName, lastName);
        if (regResult.Err) {
          throw new Error(
            "Registration failed: " + JSON.stringify(regResult.Err),
          );
        }

        setIsRegistered(true);
        const userProfile = await actor.gt_profile();
        if (userProfile.Ok) {
          setProfile(userProfile.Ok);
        }

        toast({
          title: "Profile Created",
          description: "Your account has been initialized.",
          status: "success",
          duration: 5000,
          isClosable: true,
        });
      } catch (e) {
        console.error("Registration flow error", e);
        toast({
          title: "Setup Failed",
          description: e.message || "An unexpected error occurred.",
          status: "error",
          duration: 5000,
          isClosable: true,
        });
      }
      setLoading(false);
    }
  }

  const renderContent = () => {
    if (loading) {
      return (
        <Flex minH="100vh" align="center" justify="center" bg="gray.50">
          <Spinner size="xl" color="brand.500" thickness="4px" />
        </Flex>
      );
    }

    if (!isAuthenticated) {
      return <LandingPage onLogin={login} />;
    }

    if (!isRegistered) {
      return <Onboarding onSubmit={handleRegister} isLoading={loading} />;
    }

    return <Dashboard profile={profile} actor={actor} onLogout={logout} />;
  };

  return <ChakraProvider theme={theme}>{renderContent()}</ChakraProvider>;
}

export default App;
