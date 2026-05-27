import Header from "./components/Header";
import Hero from "./components/Hero";
import Features from "./components/Features";
import PlatformShowcase from "./components/PlatformShowcase";
import InstallSection from "./components/InstallSection";
import Footer from "./components/Footer";
import "./app.css";

export default function App() {
  return (
    <>
      <Header />
      <main>
        <Hero />
        <Features />
        <PlatformShowcase />
        <InstallSection />
      </main>
      <Footer />
    </>
  );
}
