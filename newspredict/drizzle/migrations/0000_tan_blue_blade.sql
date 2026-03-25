CREATE TABLE "comments" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"market_id" uuid NOT NULL,
	"user_id" uuid NOT NULL,
	"content" text NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "comment_content_length" CHECK (length("comments"."content") > 0 AND length("comments"."content") <= 2000)
);
--> statement-breakpoint
CREATE TABLE "follows" (
	"follower_id" uuid NOT NULL,
	"following_id" uuid NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "follows_follower_id_following_id_pk" PRIMARY KEY("follower_id","following_id"),
	CONSTRAINT "no_self_follow" CHECK ("follows"."follower_id" != "follows"."following_id")
);
--> statement-breakpoint
CREATE TABLE "leaderboard_snapshots" (
	"id" integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY (sequence name "leaderboard_snapshots_id_seq" INCREMENT BY 1 MINVALUE 1 MAXVALUE 2147483647 START WITH 1 CACHE 1),
	"period" text NOT NULL,
	"rankings" jsonb NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "markets" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"question" text NOT NULL,
	"description" text,
	"category" text NOT NULL,
	"image_url" text,
	"status" text DEFAULT 'open' NOT NULL,
	"creator_id" uuid,
	"creator_type" text DEFAULT 'system' NOT NULL,
	"yes_shares" numeric(18, 4) DEFAULT '0' NOT NULL,
	"no_shares" numeric(18, 4) DEFAULT '0' NOT NULL,
	"liquidity_param" numeric(8, 2) DEFAULT '100.00' NOT NULL,
	"volume" numeric(18, 2) DEFAULT '0' NOT NULL,
	"trader_count" integer DEFAULT 0 NOT NULL,
	"version" integer DEFAULT 0 NOT NULL,
	"resolution_criteria" text NOT NULL,
	"resolution_source" text,
	"resolution_result" text,
	"resolved_at" timestamp with time zone,
	"close_at" timestamp with time zone NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"crucix_seed_id" text,
	"related_sources" text[],
	"tags" text[],
	CONSTRAINT "markets_crucix_seed_id_unique" UNIQUE("crucix_seed_id"),
	CONSTRAINT "market_status_valid" CHECK ("markets"."status" IN ('open', 'closed', 'resolved', 'cancelled')),
	CONSTRAINT "market_category_valid" CHECK ("markets"."category" IN ('geopolitics', 'economics', 'science', 'technology', 'health', 'environment', 'sports', 'entertainment', 'politics', 'other')),
	CONSTRAINT "market_creator_type_valid" CHECK ("markets"."creator_type" IN ('system', 'user', 'crucix'))
);
--> statement-breakpoint
CREATE TABLE "positions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"user_id" uuid NOT NULL,
	"market_id" uuid NOT NULL,
	"side" text NOT NULL,
	"shares" numeric(18, 4) NOT NULL,
	"avg_price" numeric(8, 4) NOT NULL,
	"realized_pnl" numeric(18, 2) DEFAULT '0' NOT NULL,
	CONSTRAINT "position_side_valid" CHECK ("positions"."side" IN ('YES', 'NO')),
	CONSTRAINT "position_shares_positive" CHECK ("positions"."shares" > 0)
);
--> statement-breakpoint
CREATE TABLE "trades" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"user_id" uuid NOT NULL,
	"market_id" uuid NOT NULL,
	"side" text NOT NULL,
	"shares" numeric(18, 4) NOT NULL,
	"price" numeric(8, 4) NOT NULL,
	"cost" numeric(18, 2) NOT NULL,
	"tx_hash" text,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "trade_side_valid" CHECK ("trades"."side" IN ('YES', 'NO')),
	CONSTRAINT "trade_shares_positive" CHECK ("trades"."shares" > 0)
);
--> statement-breakpoint
CREATE TABLE "users" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"email" text,
	"name" text,
	"avatar_url" text,
	"wallet_address" text,
	"balance" numeric(18, 2) DEFAULT '1000.00' NOT NULL,
	"total_trades" integer DEFAULT 0 NOT NULL,
	"wins" integer DEFAULT 0 NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "users_email_unique" UNIQUE("email"),
	CONSTRAINT "users_wallet_address_unique" UNIQUE("wallet_address"),
	CONSTRAINT "balance_non_negative" CHECK ("users"."balance" >= 0)
);
--> statement-breakpoint
ALTER TABLE "comments" ADD CONSTRAINT "comments_market_id_markets_id_fk" FOREIGN KEY ("market_id") REFERENCES "public"."markets"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "comments" ADD CONSTRAINT "comments_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "follows" ADD CONSTRAINT "follows_follower_id_users_id_fk" FOREIGN KEY ("follower_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "follows" ADD CONSTRAINT "follows_following_id_users_id_fk" FOREIGN KEY ("following_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "markets" ADD CONSTRAINT "markets_creator_id_users_id_fk" FOREIGN KEY ("creator_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "positions" ADD CONSTRAINT "positions_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "positions" ADD CONSTRAINT "positions_market_id_markets_id_fk" FOREIGN KEY ("market_id") REFERENCES "public"."markets"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "trades" ADD CONSTRAINT "trades_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "trades" ADD CONSTRAINT "trades_market_id_markets_id_fk" FOREIGN KEY ("market_id") REFERENCES "public"."markets"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "positions_user_market_side" ON "positions" USING btree ("user_id","market_id","side");