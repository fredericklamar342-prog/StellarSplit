import { Controller, Post, Body, Get, Param, Logger, UseFilters, BadRequestException } from '@nestjs/common';
import { UploadService } from './upload.service';
import { ApiTags, ApiOperation, ApiResponse } from '@nestjs/swagger';

@ApiTags('uploads')
@Controller('uploads')
export class UploadController {
    private readonly logger = new Logger(UploadController.name);

    constructor(private readonly uploadService: UploadService) { }

    @Post('presigned-url')
    @ApiOperation({ summary: 'Get a presigned URL for file upload' })
    @ApiResponse({ status: 201, description: 'Presigned URL generated successfully' })
    async getPresignedUrl(
        @Body() body: { fileName: string; contentType: string; fileSize?: number },
    ) {
        if (!body.fileName || !body.contentType) {
            throw new BadRequestException('fileName and contentType are required');
        }
        this.logger.log(`Requesting presigned upload URL for ${body.fileName}`);
        return await this.uploadService.getPresignedUploadUrl(body.fileName, body.contentType, body.fileSize);
    }

    @Get('download-url/:encodedKey')
    @ApiOperation({ summary: 'Get a presigned URL for file download' })
    @ApiResponse({ status: 200, description: 'Presigned URL generated successfully' })
    async getDownloadUrl(@Param('encodedKey') encodedKey: string) {
        try {
            const key = Buffer.from(encodedKey, 'base64').toString('utf-8');
            // Basic validation to ensure it's a receipts key
            if (!key.startsWith('receipts/')) {
                throw new BadRequestException('Invalid key');
            }
            this.logger.log(`Requesting presigned download URL for key ${key}`);
            const url = await this.uploadService.getPresignedDownloadUrl(key);
            return { url };
        } catch (error) {
            throw new BadRequestException('Invalid encoded key');
        }
    }
}
