using System;
using Microsoft.EntityFrameworkCore.Migrations;
using Npgsql.EntityFrameworkCore.PostgreSQL.Metadata;

#nullable disable

namespace scenario.Migrations
{
    /// <inheritdoc />
    public partial class Init : Migration
    {
        /// <inheritdoc />
        protected override void Up(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.AlterDatabase()
                .Annotation("Npgsql:Enum:compat_mood", "happy,sad");

            migrationBuilder.CreateTable(
                name: "compat_items",
                columns: table => new
                {
                    Id = table.Column<int>(type: "integer", nullable: false)
                        .Annotation("Npgsql:ValueGenerationStrategy", NpgsqlValueGenerationStrategy.IdentityByDefaultColumn),
                    Name = table.Column<string>(type: "character varying(100)", maxLength: 100, nullable: false),
                    Price = table.Column<decimal>(type: "numeric(10,2)", precision: 10, scale: 2, nullable: false),
                    Mood = table.Column<Mood>(type: "compat_mood", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_compat_items", x => x.Id);
                });

            migrationBuilder.CreateTable(
                name: "compat_orders",
                columns: table => new
                {
                    Id = table.Column<int>(type: "integer", nullable: false)
                        .Annotation("Npgsql:ValueGenerationStrategy", NpgsqlValueGenerationStrategy.IdentityByDefaultColumn),
                    ItemId = table.Column<int>(type: "integer", nullable: false),
                    Quantity = table.Column<int>(type: "integer", nullable: false),
                    Added = table.Column<DateTime>(type: "timestamp with time zone", nullable: false)
                },
                constraints: table =>
                {
                    table.PrimaryKey("PK_compat_orders", x => x.Id);
                    table.ForeignKey(
                        name: "FK_compat_orders_compat_items_ItemId",
                        column: x => x.ItemId,
                        principalTable: "compat_items",
                        principalColumn: "Id",
                        onDelete: ReferentialAction.Cascade);
                });

            migrationBuilder.CreateIndex(
                name: "IX_compat_items_Name",
                table: "compat_items",
                column: "Name",
                unique: true);

            migrationBuilder.CreateIndex(
                name: "IX_compat_orders_ItemId",
                table: "compat_orders",
                column: "ItemId");
        }

        /// <inheritdoc />
        protected override void Down(MigrationBuilder migrationBuilder)
        {
            migrationBuilder.DropTable(
                name: "compat_orders");

            migrationBuilder.DropTable(
                name: "compat_items");
        }
    }
}
