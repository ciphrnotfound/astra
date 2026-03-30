#!/bin/bash

echo "🚀 Astra Supabase Setup Script"
echo "================================"
echo ""

# Check if .env.local exists
if [ -f .env.local ]; then
    echo "⚠️  .env.local already exists. Backing up to .env.local.backup"
    cp .env.local .env.local.backup
fi

# Copy example env file
echo "📝 Creating .env.local from template..."
cp .env.local.example .env.local

echo ""
echo "✅ .env.local created!"
echo ""
echo "📋 Next steps:"
echo "1. Edit .env.local and add your Supabase credentials"
echo "2. Run: pnpm install"
echo "3. Run: pnpm db:generate"
echo "4. Run: pnpm db:migrate"
echo "5. Run: pnpm dev"
echo ""
echo "📖 See SUPABASE_SETUP.md for detailed instructions"
