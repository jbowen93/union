// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
module.exports = {
  sidebar: [
    "intro",
    {
      type: "category",
      label: "Concepts",
      items: [
        {
          type: "doc",
          id: "concepts/consensus-verification",
        },
        {
          type: "doc",
          id: "concepts/ibc",
        },
        {
          type: "doc",
          id: "concepts/permissionless-vs-trustless",
        },
      ],
    },
    {
      type: "category",
      label: "Architecture",
      items: [
        {
          type: "autogenerated",
          dirName: "architecture",
        },
      ],
    },
    {
      type: "category",
      label: "Validators",
      items: [
        {
          type: "doc",
          id: "validators/getting-started",
        },
        {
          type: "doc",
          id: "validators/docker-compose",
        },
        {
          type: "doc",
          id: "validators/nixos",
        },
        {
          type: "doc",
          id: "validators/kubernetes",
        },
      ],
    },
    {
      type: "category",
      label: "Tutorials",
      items: [
        {
          type: "doc",
          id: "tutorials/connect-two-ibc-modules",
        },
        {
          type: "doc",
          id: "tutorials/relayer-configuration",
        }
      ],
    },
    {
      type: "category",
      label: "Configurations",
      items: [
        {
          type: "doc",
          id: "configurations/ibc-parameters",
        },
      ],
    },
  ],
};
