import type { Icon } from "@astrojs/starlight/components";

type IconProps = Parameters<typeof Icon>[0];

export const socialLinks: Array<{
  href: string;
  icon: IconProps["name"];
  cta: string;
}> = [
  {
    href: "https://x.com/union_build",
    icon: "x.com",
    cta: "Join us on X",
  },
  {
    href: "https://discord.union.build",
    icon: "discord",
    cta: "Join our Discord",
  },
  {
    href: "https://x.com/union_build",
    icon: "seti:csv",
    cta: "Join our Events",
  },
];
