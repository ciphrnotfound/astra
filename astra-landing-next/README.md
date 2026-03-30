# Astra Landing Page

Clean, minimal landing page for Astra - a cross-language code migration CLI tool.

## Features

- Modern Next.js 15 with App Router
- Tailwind CSS v4
- Clean, minimal design inspired by Vercel/Linear/Resend
- Fully responsive
- Documentation site with sidebar navigation
- Dashboard with sidebar navigation
- Authentication pages (Sign In/Sign Up)
- 404 page

## Getting Started

### Prerequisites

- Node.js 18+ 
- npm or yarn

### Installation

```bash
npm install
# or
yarn install
```

### Development

```bash
npm run dev
# or
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) to view the site.

### Build

```bash
npm run build
# or
yarn build
```

## Project Structure

```
astra-landing-next/
├── app/                    # Next.js app directory
│   ├── about/             # About page
│   ├── contact/           # Contact page
│   ├── dashboard/         # Dashboard with sidebar
│   ├── docs/              # Documentation with sidebar
│   ├── research/          # Research page
│   ├── signin/            # Sign in page
│   └── signup/            # Sign up page
├── components/            # React components
│   ├── auth/             # Authentication components
│   ├── dashboard/        # Dashboard components
│   └── docs/             # Documentation components
└── public/               # Static assets
```

## Design System

- **Colors**: Cream background (#faf9f6), white sections, black/gray text
- **Fonts**: 
  - Cabinet Grotesk (headings, medium weight)
  - DM Sans (body text)
  - Space Grotesk (logo text)
- **Buttons**: Sharp corners, slide-up/slide-right animations
- **Style**: Minimal, clean, monochrome

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

Created by Shay Jeremy
