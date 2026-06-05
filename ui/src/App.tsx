import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createBrowserRouter } from "react-router";
import { ThemeProvider } from "@/theme/ThemeProvider";
import Layout from "./routes/_layout";
import IndexRoute from "./routes/index";
import ProjectRoute from "./routes/projects.$prefix";
import IssueRoute from "./routes/projects.$prefix.issues.$key";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: true,
      staleTime: 5_000,
      retry: 1,
    },
  },
});

const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { path: "/", element: <IndexRoute /> },
      { path: "/p/:prefix", element: <ProjectRoute /> },
      { path: "/p/:prefix/i/:key", element: <IssueRoute /> },
    ],
  },
]);

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <RouterProvider router={router} />
      </ThemeProvider>
    </QueryClientProvider>
  );
}
